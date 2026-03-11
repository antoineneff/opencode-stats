use std::str::FromStr;

use anyhow::anyhow;
use clap::ValueEnum;
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl FromStr for ThemeMode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            other => Err(anyhow!(
                "unsupported theme '{other}', expected dark or light"
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub foreground: Color,
    pub card_background: Color,
    pub card_border: Color,
    pub card_shadow: Color,
    pub muted: Color,
    pub accent: Color,
    pub comparison: Color,
    pub tab_active_fg: Color,
    pub tab_active_bg: Color,
    pub heat_0: Color,
    pub heat_3: Color,
    pub model_series: [Color; 12],
}

impl Theme {
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self {
                foreground: Color::Rgb(229, 233, 240),
                card_background: Color::Rgb(28, 33, 43),
                card_border: Color::Rgb(120, 130, 155),
                card_shadow: Color::Rgb(0, 0, 0),
                muted: Color::Rgb(128, 134, 152),
                accent: Color::Rgb(136, 192, 208),
                comparison: Color::Rgb(180, 190, 254),
                tab_active_fg: Color::Black,
                tab_active_bg: Color::Rgb(136, 192, 208),
                heat_0: Color::Rgb(94, 98, 115),
                heat_3: Color::Rgb(136, 192, 208),
                model_series: [
                    Color::Rgb(191, 97, 106),
                    Color::Rgb(208, 135, 112),
                    Color::Rgb(235, 203, 139),
                    Color::Rgb(163, 190, 140),
                    Color::Rgb(136, 192, 208),
                    Color::Rgb(129, 161, 193),
                    Color::Rgb(180, 142, 173),
                    Color::Rgb(171, 121, 103),
                    Color::Rgb(94, 129, 172),
                    Color::Rgb(143, 188, 187),
                    Color::Rgb(216, 222, 233),
                    Color::Rgb(76, 86, 106),
                ],
            },
            ThemeMode::Light => Self {
                foreground: Color::Rgb(37, 41, 51),
                card_background: Color::Rgb(252, 253, 255),
                card_border: Color::Rgb(173, 183, 201),
                card_shadow: Color::Rgb(96, 107, 128),
                muted: Color::Rgb(90, 98, 115),
                accent: Color::Rgb(0, 122, 163),
                comparison: Color::Rgb(94, 92, 230),
                tab_active_fg: Color::White,
                tab_active_bg: Color::Rgb(0, 122, 163),
                heat_0: Color::Rgb(160, 170, 186),
                heat_3: Color::Rgb(0, 122, 163),
                model_series: [
                    Color::Rgb(167, 40, 40),
                    Color::Rgb(175, 94, 0),
                    Color::Rgb(145, 108, 0),
                    Color::Rgb(51, 122, 68),
                    Color::Rgb(0, 122, 163),
                    Color::Rgb(50, 88, 160),
                    Color::Rgb(126, 76, 142),
                    Color::Rgb(120, 78, 52),
                    Color::Rgb(72, 106, 154),
                    Color::Rgb(70, 150, 154),
                    Color::Rgb(34, 34, 34),
                    Color::Rgb(90, 98, 115),
                ],
            },
        }
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn comparison_style(&self) -> Style {
        Style::default()
            .fg(self.comparison)
            .add_modifier(Modifier::BOLD)
    }

    pub fn series_color(&self, index: usize) -> Color {
        self.model_series[index % self.model_series.len()]
    }
}
