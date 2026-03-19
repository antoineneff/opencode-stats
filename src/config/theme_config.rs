use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use ratatui::style::Color;
use serde::Deserialize;

use crate::config;
use crate::ui::theme::{NamedTheme, Theme, ThemeKind, builtin_themes};

#[derive(Clone, Debug, Default)]
pub struct ThemeCatalog {
    themes: BTreeMap<String, NamedTheme>,
}

impl ThemeCatalog {
    pub fn load() -> Result<Self> {
        let mut themes = builtin_themes();

        if let Some(index_path) = config::themes_index_path()
            && index_path.exists()
        {
            let index = read_toml::<ThemesIndexFile>(&index_path)?;
            for entry in index.theme {
                let key = normalize_theme_name(&entry.name)?;
                themes.insert(
                    key,
                    NamedTheme {
                        kind: entry.theme.kind,
                        theme: entry.theme.into_runtime_theme()?,
                    },
                );
            }
        }

        if let Some(theme_dir) = config::themes_dir_path()
            && theme_dir.exists()
        {
            let mut paths = std::fs::read_dir(&theme_dir)
                .with_context(|| format!("failed to read {}", theme_dir.display()))?
                .collect::<std::io::Result<Vec<_>>>()
                .with_context(|| format!("failed to list {}", theme_dir.display()))?
                .into_iter()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml")
                })
                .collect::<Vec<_>>();
            paths.sort();

            for path in paths {
                let key = derive_theme_name_from_path(&path)?;
                let file_theme = read_toml::<ThemeContent>(&path)?;
                themes.insert(
                    key,
                    NamedTheme {
                        kind: file_theme.kind,
                        theme: file_theme.into_runtime_theme()?,
                    },
                );
            }
        }

        Ok(Self { themes })
    }

    pub fn get(&self, name: &str) -> Option<&NamedTheme> {
        self.themes.get(&name.trim().to_ascii_lowercase())
    }

    pub fn names(&self) -> Vec<&str> {
        self.themes.keys().map(String::as_str).collect()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ThemesIndexFile {
    theme: Vec<ThemeEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeEntry {
    name: String,
    #[serde(flatten)]
    theme: ThemeContent,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeContent {
    #[serde(rename = "type")]
    kind: ThemeKind,
    base: BasePalette,
    card: CardPalette,
    accent: AccentPalette,
    tab: TabPalette,
    heatmap: HeatmapPalette,
    series: SeriesPalette,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BasePalette {
    foreground: String,
    muted: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CardPalette {
    background: String,
    border: String,
    shadow: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccentPalette {
    primary: String,
    comparison: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TabPalette {
    active_fg: String,
    active_bg: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HeatmapPalette {
    empty: String,
    active: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeriesPalette {
    model: Vec<String>,
}

impl ThemeContent {
    fn into_runtime_theme(self) -> Result<Theme> {
        Ok(Theme {
            foreground: parse_hex_color(&self.base.foreground)?,
            card_background: parse_hex_color(&self.card.background)?,
            card_border: parse_hex_color(&self.card.border)?,
            card_shadow: parse_hex_color(&self.card.shadow)?,
            muted: parse_hex_color(&self.base.muted)?,
            accent: parse_hex_color(&self.accent.primary)?,
            comparison: parse_hex_color(&self.accent.comparison)?,
            tab_active_fg: parse_hex_color(&self.tab.active_fg)?,
            tab_active_bg: parse_hex_color(&self.tab.active_bg)?,
            heat_0: parse_hex_color(&self.heatmap.empty)?,
            heat_3: parse_hex_color(&self.heatmap.active)?,
            model_series: parse_model_series(&self.series.model)?,
        })
    }
}

fn parse_model_series(values: &[String]) -> Result<[Color; 12]> {
    if values.len() != 12 {
        bail!(
            "series.model must contain exactly 12 colors, got {}",
            values.len()
        );
    }

    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        parsed.push(parse_hex_color(value)?);
    }

    parsed
        .try_into()
        .map_err(|_| anyhow!("failed to parse series.model colors"))
}

fn parse_hex_color(value: &str) -> Result<Color> {
    let raw = value.trim();
    let hex = raw
        .strip_prefix('#')
        .ok_or_else(|| anyhow!("invalid color '{raw}', expected #RRGGBB"))?;

    if hex.len() != 6 {
        bail!("invalid color '{raw}', expected #RRGGBB");
    }

    let parse = |range: std::ops::Range<usize>| -> Result<u8> {
        u8::from_str_radix(&hex[range], 16)
            .map_err(|_| anyhow!("invalid color '{raw}', expected #RRGGBB"))
    };

    Ok(Color::Rgb(parse(0..2)?, parse(2..4)?, parse(4..6)?))
}

fn normalize_theme_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("theme name cannot be empty");
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn derive_theme_name_from_path(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| anyhow!("invalid theme filename {}", path.display()))?;
    normalize_theme_name(stem)
}

fn read_toml<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{ThemeContent, ThemeKind, parse_hex_color};
    use ratatui::style::Color;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color("#010203").unwrap(), Color::Rgb(1, 2, 3));
        assert!(parse_hex_color("010203").is_err());
    }

    #[test]
    fn parses_theme_content() {
        let value = toml::from_str::<ThemeContent>(
            r##"
type = "dark"

[base]
foreground = "#E5E9F0"
muted = "#808698"

[card]
background = "#1C212B"
border = "#78829B"
shadow = "#000000"

[accent]
primary = "#88C0D0"
comparison = "#B4BEFE"

[tab]
active_fg = "#000000"
active_bg = "#88C0D0"

[heatmap]
empty = "#5E6273"
active = "#88C0D0"

[series]
model = ["#BF616A", "#D08770", "#EBCB8B", "#A3BE8C", "#88C0D0", "#81A1C1", "#B48EAD", "#AB7967", "#5E81AC", "#8FBCBB", "#D8DEE9", "#4C566A"]
"##,
        )
        .unwrap();

        assert_eq!(value.kind, ThemeKind::Dark);
        assert_eq!(
            value.into_runtime_theme().unwrap().foreground,
            Color::Rgb(229, 233, 240)
        );
    }
}
