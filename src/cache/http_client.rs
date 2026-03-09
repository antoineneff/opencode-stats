use anyhow::{Context, Result};
use serde_json::Value;

pub async fn fetch_json(url: &str) -> Result<Value> {
    let client = reqwest::Client::builder()
        .user_agent("oc-stats/0.1")
        .build()
        .context("failed to build HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to fetch {}", url))?
        .error_for_status()
        .with_context(|| format!("request to {} failed", url))?;

    response
        .json::<Value>()
        .await
        .context("failed to decode JSON response")
}
