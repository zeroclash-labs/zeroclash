//! ZeroClash Design System — unified tokens for colors, spacing, typography, and
//! corner radii. All e gui widgets reference these instead of hardcoding values.

use egui::{Color32, CornerRadius, Stroke, Vec2};

// ── Colors ─────────────────────────────────────────────────────────────────

pub struct Colors {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_text: Color32,
    pub sidebar_active: Color32,
}

pub const LIGHT: Colors = Colors {
    bg: Color32::from_rgb(245, 247, 250),
    surface: Color32::from_rgb(255, 255, 255),
    surface_hover: Color32::from_rgb(240, 242, 245),
    border: Color32::from_rgb(226, 230, 236),
    text_primary: Color32::from_rgb(30, 41, 59),
    text_secondary: Color32::from_rgb(100, 116, 139),
    text_muted: Color32::from_rgb(148, 163, 184),
    accent: Color32::from_rgb(59, 130, 246),
    accent_dim: Color32::from_rgba_premultiplied(59, 130, 246, 25),
    success: Color32::from_rgb(34, 197, 94),
    warning: Color32::from_rgb(251, 191, 36),
    danger: Color32::from_rgb(239, 68, 68),
    sidebar_bg: Color32::from_rgb(15, 23, 42),
    sidebar_text: Color32::from_rgb(203, 213, 225),
    sidebar_active: Color32::from_rgb(59, 130, 246),
};

pub const DARK: Colors = Colors {
    bg: Color32::from_rgb(15, 23, 42),
    surface: Color32::from_rgb(30, 41, 59),
    surface_hover: Color32::from_rgb(51, 65, 85),
    border: Color32::from_rgb(51, 65, 85),
    text_primary: Color32::from_rgb(226, 232, 240),
    text_secondary: Color32::from_rgb(148, 163, 184),
    text_muted: Color32::from_rgb(100, 116, 139),
    accent: Color32::from_rgb(96, 165, 250),
    accent_dim: Color32::from_rgba_premultiplied(96, 165, 250, 25),
    success: Color32::from_rgb(74, 222, 128),
    warning: Color32::from_rgb(251, 191, 36),
    danger: Color32::from_rgb(248, 113, 113),
    sidebar_bg: Color32::from_rgb(8, 14, 26),
    sidebar_text: Color32::from_rgb(148, 163, 184),
    sidebar_active: Color32::from_rgb(96, 165, 250),
};

// ── Spacing ────────────────────────────────────────────────────────────────

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

// ── Corner radius ──────────────────────────────────────────────────────────

pub const RADIUS_SM: CornerRadius = CornerRadius::same(4);
pub const RADIUS_MD: CornerRadius = CornerRadius::same(6);
pub const RADIUS_LG: CornerRadius = CornerRadius::same(10);

// ── Typography ─────────────────────────────────────────────────────────────

pub const FONT_XS: f32 = 11.0;
pub const FONT_SM: f32 = 12.0;
pub const FONT_MD: f32 = 14.0;
pub const FONT_LG: f32 = 18.0;
pub const FONT_XL: f32 = 24.0;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Returns true if dark mode should be used.
pub fn is_dark_mode(ctx: &egui::Context) -> bool {
    ctx.global_style().visuals.dark_mode
}

/// Get the active color palette.
pub fn palette(ctx: &egui::Context) -> &'static Colors {
    if is_dark_mode(ctx) { &DARK } else { &LIGHT }
}

/// Create a standard card frame.
pub fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let c = palette(ui.ctx());
    egui::Frame::default()
        .fill(c.surface)
        .corner_radius(RADIUS_MD)
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(Vec2::new(SPACE_LG, SPACE_MD))
        .outer_margin(Vec2::new(0.0, SPACE_SM))
}

/// Create a subtle section title.
pub fn section_title(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(egui::RichText::new(text).size(FONT_XS).color(c.text_muted).strong());
}

/// Standard page heading.
pub fn page_heading(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(egui::RichText::new(text).size(FONT_XL).color(c.text_primary).strong());
}

/// Muted secondary text.
pub fn muted(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(egui::RichText::new(text).size(FONT_SM).color(c.text_muted));
}

/// Compact badge pill.
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let _c = palette(ui.ctx());
    // Use a simple label with background-like styling
    ui.label(
        egui::RichText::new(format!(" {text} "))
            .size(FONT_XS)
            .color(color),
    );
}
