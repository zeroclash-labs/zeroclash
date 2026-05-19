//! ZeroClash Design System — unified tokens for colors, spacing, typography, and
//! corner radii. All egui widgets reference these instead of hardcoding values.

use egui::{Color32, CornerRadius, Stroke, Vec2};

// ── Colors ─────────────────────────────────────────────────────────────────

pub struct Colors {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub surface_hover: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub accent_glow: Color32,
    pub success: Color32,
    pub success_dim: Color32,
    pub warning: Color32,
    pub warning_dim: Color32,
    pub danger: Color32,
    pub danger_dim: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_text: Color32,
    pub sidebar_text_muted: Color32,
    pub sidebar_active_bg: Color32,
    pub input_bg: Color32,
}

pub const LIGHT: Colors = Colors {
    bg: Color32::from_rgb(245, 247, 250),
    surface: Color32::from_rgb(255, 255, 255),
    surface_alt: Color32::from_rgb(248, 250, 252),
    surface_hover: Color32::from_rgb(240, 242, 245),
    border: Color32::from_rgb(226, 230, 236),
    divider: Color32::from_rgb(241, 243, 248),
    text_primary: Color32::from_rgb(15, 23, 42),
    text_secondary: Color32::from_rgb(71, 85, 105),
    text_muted: Color32::from_rgb(148, 163, 184),
    accent: Color32::from_rgb(59, 130, 246),
    accent_dim: Color32::from_rgba_premultiplied(59, 130, 246, 25),
    accent_glow: Color32::from_rgba_premultiplied(59, 130, 246, 40),
    success: Color32::from_rgb(34, 197, 94),
    success_dim: Color32::from_rgba_premultiplied(34, 197, 94, 20),
    warning: Color32::from_rgb(251, 191, 36),
    warning_dim: Color32::from_rgba_premultiplied(251, 191, 36, 20),
    danger: Color32::from_rgb(239, 68, 68),
    danger_dim: Color32::from_rgba_premultiplied(239, 68, 68, 20),
    sidebar_bg: Color32::from_rgb(15, 23, 42),
    sidebar_text: Color32::from_rgb(203, 213, 225),
    sidebar_text_muted: Color32::from_rgb(100, 116, 139),
    sidebar_active_bg: Color32::from_rgba_premultiplied(59, 130, 246, 30),
    input_bg: Color32::from_rgb(241, 243, 248),
};

pub const DARK: Colors = Colors {
    bg: Color32::from_rgb(10, 16, 30),
    surface: Color32::from_rgb(20, 30, 48),
    surface_alt: Color32::from_rgb(25, 36, 56),
    surface_hover: Color32::from_rgb(35, 48, 70),
    border: Color32::from_rgb(40, 55, 80),
    divider: Color32::from_rgba_premultiplied(148, 163, 184, 25),
    text_primary: Color32::from_rgb(226, 232, 240),
    text_secondary: Color32::from_rgb(148, 163, 184),
    text_muted: Color32::from_rgb(100, 116, 139),
    accent: Color32::from_rgb(96, 165, 250),
    accent_dim: Color32::from_rgba_premultiplied(96, 165, 250, 25),
    accent_glow: Color32::from_rgba_premultiplied(96, 165, 250, 50),
    success: Color32::from_rgb(74, 222, 128),
    success_dim: Color32::from_rgba_premultiplied(74, 222, 128, 20),
    warning: Color32::from_rgb(251, 191, 36),
    warning_dim: Color32::from_rgba_premultiplied(251, 191, 36, 25),
    danger: Color32::from_rgb(248, 113, 113),
    danger_dim: Color32::from_rgba_premultiplied(248, 113, 113, 20),
    sidebar_bg: Color32::from_rgb(6, 12, 22),
    sidebar_text: Color32::from_rgb(203, 213, 225),
    sidebar_text_muted: Color32::from_rgb(100, 116, 139),
    sidebar_active_bg: Color32::from_rgba_premultiplied(96, 165, 250, 35),
    input_bg: Color32::from_rgb(15, 24, 38),
};

// ── Spacing ────────────────────────────────────────────────────────────────

pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const SPACE_XXL: f32 = 32.0;

// ── Corner radius ──────────────────────────────────────────────────────────

pub const RADIUS_SM: CornerRadius = CornerRadius::same(4);
pub const RADIUS_MD: CornerRadius = CornerRadius::same(6);
pub const RADIUS_LG: CornerRadius = CornerRadius::same(10);
pub const RADIUS_XL: CornerRadius = CornerRadius::same(14);

// ── Typography ─────────────────────────────────────────────────────────────

pub const FONT_XS: f32 = 11.0;
pub const FONT_SM: f32 = 12.0;
pub const FONT_MD: f32 = 14.0;
pub const FONT_LG: f32 = 18.0;
pub const FONT_XL: f32 = 24.0;
pub const FONT_XXL: f32 = 32.0;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Returns true if dark mode should be used.
pub fn is_dark_mode(ctx: &egui::Context) -> bool {
    ctx.global_style().visuals.dark_mode
}

/// Get the active color palette.
pub fn palette(ctx: &egui::Context) -> &'static Colors {
    if is_dark_mode(ctx) { &DARK } else { &LIGHT }
}

/// Create a standard card frame with subtle depth.
pub fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let c = palette(ui.ctx());
    egui::Frame::default()
        .fill(c.surface)
        .corner_radius(RADIUS_MD)
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(Vec2::new(SPACE_LG, SPACE_LG))
        .outer_margin(Vec2::new(0.0, SPACE_SM))
}

/// Create a compact card frame (tighter padding).
pub fn compact_card_frame(ui: &egui::Ui) -> egui::Frame {
    let c = palette(ui.ctx());
    egui::Frame::default()
        .fill(c.surface)
        .corner_radius(RADIUS_SM)
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(Vec2::new(SPACE_MD, SPACE_SM))
        .outer_margin(Vec2::new(0.0, SPACE_XS))
}

/// Create a frame with an accent left border for highlighted sections.
pub fn accent_left_frame(ui: &egui::Ui) -> egui::Frame {
    let c = palette(ui.ctx());
    egui::Frame::default()
        .fill(c.surface)
        .corner_radius(RADIUS_MD)
        .stroke(Stroke::new(1.0, c.border))
        .inner_margin(Vec2::new(SPACE_LG, SPACE_LG))
        .outer_margin(Vec2::new(0.0, SPACE_SM))
}

/// Draw an accent left border on the last-painted rect.
pub fn paint_accent_left_border(ui: &egui::Ui, rect: egui::Rect) {
    let c = palette(ui.ctx());
    let bar = egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
    ui.painter().rect_filled(bar, 1.5, c.accent);
}

/// Create a section title label.
pub fn section_title(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(
        egui::RichText::new(text)
            .size(FONT_XS)
            .color(c.text_muted)
            .strong(),
    );
}

/// Standard page heading.
pub fn page_heading(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(
        egui::RichText::new(text)
            .size(FONT_XL)
            .color(c.text_primary)
            .strong(),
    );
}

/// Muted secondary text.
pub fn muted(ui: &mut egui::Ui, text: &str) {
    let c = palette(ui.ctx());
    ui.label(egui::RichText::new(text).size(FONT_SM).color(c.text_muted));
}

/// Compact badge pill with filled background.
pub fn badge(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    egui::Frame::default()
        .fill(bg)
        .corner_radius(RADIUS_SM)
        .inner_margin(egui::vec2(SPACE_XS + 2.0, SPACE_XXS + 1.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(FONT_XS).color(fg));
        });
}

/// A subtle horizontal divider (custom-painted line).
pub fn divider(ui: &mut egui::Ui) {
    let c = palette(ui.ctx());
    ui.add_space(SPACE_XS);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, c.divider);
    ui.add_space(SPACE_XS);
}

/// Style a label as a colored status dot.
pub fn status_dot(ui: &mut egui::Ui, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let center = rect.center();
    ui.painter().circle_filled(center, size * 0.5, color);
}
