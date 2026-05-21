use gpui::Global;

use crate::design::{self, Colors};

pub struct Theme {
    pub colors: Colors,
    pub mode: ThemeMode,
}

impl Global for Theme {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Theme {
    pub fn detect() -> Self {
        let mode = match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => ThemeMode::Dark,
            _ => ThemeMode::Light,
        };
        Self::from_mode(mode)
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Light => design::light(),
            ThemeMode::Dark => design::dark(),
        };
        Self { colors, mode }
    }

    pub fn parse_theme(s: &str) -> Self {
        match s {
            "dark" => Self::from_mode(ThemeMode::Dark),
            _ => Self::from_mode(ThemeMode::Light),
        }
    }
}

pub fn init_theme(cx: &mut gpui::App, mode: &str) {
    let theme = if mode.is_empty() {
        Theme::detect()
    } else {
        Theme::parse_theme(mode)
    };
    cx.set_global(theme);
}
