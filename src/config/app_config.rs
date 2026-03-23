use serde::Deserialize;

use crate::config;
use crate::config::errors::{Error, Result};
use crate::ui::theme::ThemeMode;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub theme: ThemeConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeConfig {
    pub default: ThemeMode,
    pub dark: String,
    pub light: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            default: ThemeMode::Auto,
            dark: "dark".to_string(),
            light: "light".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let Some(path) = config::config_path() else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path).map_err(|e| Error::config_read(&path, e))?;
        let parsed = toml::from_str(&contents).map_err(|e| Error::config_parse(&path, e))?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ThemeConfig};
    use crate::ui::theme::ThemeMode;

    #[test]
    fn defaults_match_builtin_theme_names() {
        let config = AppConfig::default();
        assert_eq!(config.theme.default, ThemeMode::Auto);
        assert_eq!(config.theme.dark, "dark");
        assert_eq!(config.theme.light, "light");
    }

    #[test]
    fn parses_theme_block() {
        let parsed = toml::from_str::<AppConfig>(
            r#"
[theme]
default = "light"
dark = "nord"
light = "paper"
"#,
        )
        .unwrap();

        assert_eq!(parsed.theme.default, ThemeMode::Light);
        assert_eq!(parsed.theme.dark, "nord");
        assert_eq!(parsed.theme.light, "paper");
    }

    #[test]
    fn allows_missing_theme_block() {
        let parsed = toml::from_str::<AppConfig>("").unwrap();
        assert_eq!(parsed.theme, ThemeConfig::default());
    }

    #[test]
    fn rejects_unknown_fields() {
        let parsed = toml::from_str::<AppConfig>(
            r#"
[theme]
default = "auto"
dark = "dark"
light = "light"
extra = "nope"
"#,
        )
        .unwrap_err();
        assert!(format!("{parsed:#}").contains("unknown field"));
    }
}
