use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::cache::http_client;
use crate::db::models::{TokenUsage, UsageEvent};

const BUNDLED_MODELS_JSON: &str = include_str!("../../ref/ocmonitor-share/ocmonitor/models.json");
const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CACHE_TTL_SECS: u64 = 60 * 60;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input: Decimal,
    pub output: Decimal,
    #[serde(rename = "cacheWrite", default)]
    pub cache_write: Decimal,
    #[serde(rename = "cacheRead", default)]
    pub cache_read: Decimal,
    #[serde(rename = "contextWindow", default)]
    pub context_window: u64,
    #[serde(rename = "sessionQuota", default)]
    pub session_quota: Decimal,
}

impl ModelPricing {
    pub fn with_fallbacks(mut self) -> Self {
        if self.cache_write.is_zero() {
            self.cache_write = self.input;
        }
        if self.cache_read.is_zero() {
            self.cache_read = self.input * Decimal::new(1, 1);
        }
        self
    }
}

#[derive(Clone, Debug)]
pub struct PricingCatalog {
    pub models: BTreeMap<String, ModelPricing>,
    pub cache_path: PathBuf,
    pub refresh_needed: bool,
}

impl PricingCatalog {
    pub fn load() -> Result<Self> {
        let bundled = load_bundled_models()?;
        let cache_path = default_cache_path()?;
        let cached = load_cached_models(&cache_path).unwrap_or_default();
        let refresh_needed = cache_is_stale(&cache_path).unwrap_or(true);

        let mut merged = bundled.clone();
        merge_fill_only(&mut merged, cached);

        Ok(Self {
            models: merged,
            cache_path,
            refresh_needed,
        })
    }

    pub fn lookup(&self, model_id: &str) -> Option<&ModelPricing> {
        let lowercase = model_id.to_lowercase();
        if let Some(value) = self.models.get(&lowercase) {
            return Some(value);
        }

        let normalized = normalize_model_key(model_id);
        if let Some(value) = self.models.get(&normalized) {
            return Some(value);
        }

        if let Some((_, bare)) = lowercase.split_once('/') {
            let normalized_bare = normalize_model_key(bare);
            if let Some(value) = self.models.get(&normalized_bare) {
                return Some(value);
            }
        }

        None
    }

    pub fn cost_for_event(&self, event: &UsageEvent) -> Decimal {
        if let Some(cost) = event.stored_cost_usd {
            if cost > Decimal::ZERO {
                return cost;
            }
        }

        let Some(pricing) = self.lookup(&event.model_id) else {
            return Decimal::ZERO;
        };
        price_tokens(&event.tokens, pricing)
    }

}

pub fn price_tokens(tokens: &TokenUsage, pricing: &ModelPricing) -> Decimal {
    let million = Decimal::from(1_000_000u64);
    (Decimal::from(tokens.input) * pricing.input
        + Decimal::from(tokens.output) * pricing.output
        + Decimal::from(tokens.cache_write) * pricing.cache_write
        + Decimal::from(tokens.cache_read) * pricing.cache_read)
        / million
}

pub fn default_cache_path() -> Result<PathBuf> {
    let Some(config_dir) = dirs::config_dir() else {
        anyhow::bail!("could not determine config directory");
    };
    Ok(config_dir.join("oc-stats").join("models.json"))
}

pub async fn refresh_remote_models(cache_path: PathBuf, sender: mpsc::UnboundedSender<PricingCatalog>) {
    let fetch_result = fetch_remote_catalog(&cache_path).await;
    if let Ok(catalog) = fetch_result {
        let _ = sender.send(catalog);
    }
}

async fn fetch_remote_catalog(cache_path: &Path) -> Result<PricingCatalog> {
    let payload = http_client::fetch_json(MODELS_DEV_URL).await?;
    let remote = map_models_dev_to_local(&payload);
    persist_cached_models(cache_path, &remote)?;

    let mut bundled = load_bundled_models()?;
    merge_fill_only(&mut bundled, remote);

    Ok(PricingCatalog {
        models: bundled,
        cache_path: cache_path.to_path_buf(),
        refresh_needed: false,
    })
}

fn persist_cached_models(path: &Path, models: &BTreeMap<String, ModelPricing>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {}", parent.display()))?;
    }
    let temp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(models).context("failed to encode cached pricing")?;
    fs::write(&temp, bytes).with_context(|| format!("failed to write {}", temp.display()))?;
    fs::rename(&temp, path).with_context(|| format!("failed to move {} into place", temp.display()))?;
    Ok(())
}

fn load_bundled_models() -> Result<BTreeMap<String, ModelPricing>> {
    let raw = serde_json::from_str::<BTreeMap<String, ModelPricing>>(BUNDLED_MODELS_JSON)
        .context("failed to parse bundled model pricing")?;
    Ok(raw
        .into_iter()
        .map(|(key, value)| (key.to_lowercase(), value.with_fallbacks()))
        .collect())
}

fn load_cached_models(path: &Path) -> Result<BTreeMap<String, ModelPricing>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let raw = serde_json::from_slice::<BTreeMap<String, ModelPricing>>(&bytes)
        .context("failed to parse cached models")?;
    Ok(raw
        .into_iter()
        .map(|(key, value)| (key.to_lowercase(), value.with_fallbacks()))
        .collect())
}

fn cache_is_stale(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::from_secs(CACHE_TTL_SECS + 1));
    Ok(age.as_secs() >= CACHE_TTL_SECS)
}

fn merge_fill_only(base: &mut BTreeMap<String, ModelPricing>, extra: BTreeMap<String, ModelPricing>) {
    for (key, value) in extra {
        base.entry(key).or_insert_with(|| value.with_fallbacks());
    }
}

fn map_models_dev_to_local(payload: &serde_json::Value) -> BTreeMap<String, ModelPricing> {
    let mut result = BTreeMap::new();
    let providers = payload.get("providers").and_then(|value| value.as_object());
    let Some(providers) = providers else {
        return result;
    };

    for (provider_id, provider_data) in providers {
        let models = provider_data.get("models").and_then(|value| value.as_object());
        let Some(models) = models else {
            continue;
        };

        for (model_id, model_data) in models {
            let cost = model_data.get("cost").and_then(|value| value.as_object());
            let limit = model_data.get("limit").and_then(|value| value.as_object());

            let pricing = ModelPricing {
                input: decimal_from_json(cost.and_then(|map| map.get("prompt"))),
                output: decimal_from_json(cost.and_then(|map| map.get("completion"))),
                cache_write: decimal_from_json(cost.and_then(|map| map.get("input_cache_write"))),
                cache_read: decimal_from_json(cost.and_then(|map| map.get("input_cache_read"))),
                context_window: limit
                    .and_then(|map| map.get("context"))
                    .and_then(|value| value.as_u64())
                    .unwrap_or_default(),
                session_quota: Decimal::ZERO,
            }
            .with_fallbacks();

            let bare = model_id.to_lowercase();
            let prefixed = format!("{}/{}", provider_id.to_lowercase(), bare);
            result.entry(bare).or_insert_with(|| pricing.clone());
            result.entry(prefixed).or_insert(pricing);
        }
    }

    result
}

fn decimal_from_json(value: Option<&serde_json::Value>) -> Decimal {
    let Some(value) = value else {
        return Decimal::ZERO;
    };

    if let Some(number) = value.as_f64() {
        Decimal::try_from(number).unwrap_or(Decimal::ZERO)
    } else if let Some(number) = value.as_str() {
        number.parse::<Decimal>().unwrap_or(Decimal::ZERO)
    } else {
        Decimal::ZERO
    }
}

pub fn normalize_model_key(model_id: &str) -> String {
    let mut model = model_id.to_lowercase();
    if let Some((provider, bare)) = model.split_once('/') {
        let normalized_bare = normalize_model_key(bare);
        return format!("{provider}/{normalized_bare}");
    }

    if model.len() > 9 {
        let suffix = &model[model.len() - 9..];
        if suffix.starts_with('-') && suffix[1..].chars().all(|value| value.is_ascii_digit()) {
            model.truncate(model.len() - 9);
        }
    }

    model = regexless_replace_version(&model, "claude-opus-");
    model = regexless_replace_version(&model, "claude-sonnet-");
    model = regexless_replace_version(&model, "claude-haiku-");
    model = regexless_replace_version(&model, "gpt-");

    if let Some(stripped) = model.strip_prefix("kimi-k-") {
        if stripped.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
            model = format!("kimi-k{stripped}");
        }
    }

    model
}

fn regexless_replace_version(value: &str, prefix: &str) -> String {
    if let Some(rest) = value.strip_prefix(prefix) {
        let segments: Vec<&str> = rest.split('-').collect();
        if segments.len() >= 2
            && segments[0].chars().all(|ch| ch.is_ascii_digit())
            && segments[1].chars().all(|ch| ch.is_ascii_digit())
        {
            let merged = format!("{}.{}", segments[0], segments[1]);
            let suffix = if segments.len() > 2 {
                format!("-{}", segments[2..].join("-"))
            } else {
                String::new()
            };
            return format!("{prefix}{merged}{suffix}");
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_model_key, ModelPricing};
    use rust_decimal::Decimal;

    #[test]
    fn normalizes_date_suffixes() {
        assert_eq!(normalize_model_key("claude-sonnet-4-5-20250514"), "claude-sonnet-4.5");
        assert_eq!(normalize_model_key("gpt-5-1"), "gpt-5.1");
    }

    #[test]
    fn fills_cache_fallbacks() {
        let pricing = ModelPricing {
            input: Decimal::new(3, 0),
            output: Decimal::new(15, 0),
            cache_write: Decimal::ZERO,
            cache_read: Decimal::ZERO,
            context_window: 0,
            session_quota: Decimal::ZERO,
        }
        .with_fallbacks();

        assert_eq!(pricing.cache_write, Decimal::new(3, 0));
        assert_eq!(pricing.cache_read, Decimal::new(3, 0) * Decimal::new(1, 1));
    }
}
