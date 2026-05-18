//! Theme management for ZeroClash egui UI.
//!
//! Provides light and dark visual styles.

use egui::Visuals;

/// Create light theme visuals.
pub fn light_theme() -> Visuals {
    let mut visuals = Visuals::light();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals
}

/// Create dark theme visuals.
pub fn dark_theme() -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals
}

/// Apply a theme by name: "light", "dark", or "system".
pub fn apply_theme(ctx: &egui::Context, theme: &str) {
    let visuals = match theme {
        "dark" => dark_theme(),
        "light" => light_theme(),
        _ => {
            match dark_light::detect() {
                Ok(dark_light::Mode::Dark) => dark_theme(),
                _ => light_theme(),
            }
        }
    };
    ctx.set_visuals(visuals);
}
