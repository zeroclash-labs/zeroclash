//! Theme management for ZeroClash egui UI.
//!
//! Constructs egui Visuals that align with our design tokens.

use egui::Visuals;

/// Create light theme visuals using design token colors.
pub fn light_theme() -> Visuals {
    let mut visuals = Visuals::light();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    // Align widget bg with our surface token
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 250, 252);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(241, 243, 248);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(226, 232, 240);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(59, 130, 246, 40);
    visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(59, 130, 246, 30);
    visuals.extreme_bg_color = egui::Color32::from_rgb(232, 234, 240);
    visuals
}

/// Create dark theme visuals using design token colors.
pub fn dark_theme() -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    // Align widget bg with our surface tokens
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 36, 56);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(20, 30, 48);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(35, 48, 70);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(96, 165, 250, 40);
    visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(96, 165, 250, 35);
    visuals.extreme_bg_color = egui::Color32::from_rgb(15, 24, 38);
    visuals
}

/// Apply a theme by name: "light", "dark", or "system".
pub fn apply_theme(ctx: &egui::Context, theme: &str) {
    let visuals = match theme {
        "dark" => dark_theme(),
        "light" => light_theme(),
        _ => match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => dark_theme(),
            _ => light_theme(),
        },
    };
    ctx.set_visuals(visuals);
}
