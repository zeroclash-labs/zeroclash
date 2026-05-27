use gpui::{Hsla, rgb, rgba};

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub bg: Hsla,
    pub surface: Hsla,
    pub surface_alt: Hsla,
    pub surface_hover: Hsla,
    pub border: Hsla,
    pub divider: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub accent: Hsla,
    pub accent_dim: Hsla,
    pub accent_glow: Hsla,
    pub success: Hsla,
    pub success_dim: Hsla,
    pub warning: Hsla,
    pub warning_dim: Hsla,
    pub danger: Hsla,
    pub danger_dim: Hsla,
    pub sidebar_bg: Hsla,
    pub sidebar_text: Hsla,
    pub sidebar_text_muted: Hsla,
    pub sidebar_active_bg: Hsla,
    pub input_bg: Hsla,
}

pub fn light() -> Colors {
    Colors {
        bg: rgb(0xf5f7fa).into(),
        surface: rgb(0xffffff).into(),
        surface_alt: rgb(0xf8fafc).into(),
        surface_hover: rgb(0xf0f2f5).into(),
        border: rgb(0xe2e6ec).into(),
        divider: rgb(0xf1f3f8).into(),
        text_primary: rgb(0x0f172a).into(),
        text_secondary: rgb(0x475569).into(),
        text_muted: rgb(0x94a3b8).into(),
        accent: rgb(0x3b82f6).into(),
        accent_dim: rgba(0x3b82f619).into(),
        accent_glow: rgba(0x3b82f628).into(),
        success: rgb(0x22c55e).into(),
        success_dim: rgba(0x22c55e14).into(),
        warning: rgb(0xfbbf24).into(),
        warning_dim: rgba(0xfbbf2419).into(),
        danger: rgb(0xef4444).into(),
        danger_dim: rgba(0xef444414).into(),
        sidebar_bg: rgb(0x0f172a).into(),
        sidebar_text: rgb(0xcbd5e1).into(),
        sidebar_text_muted: rgb(0x64748b).into(),
        sidebar_active_bg: rgba(0x3b82f61e).into(),
        input_bg: rgb(0xf1f3f8).into(),
    }
}

pub fn dark() -> Colors {
    Colors {
        bg: rgb(0x0a101e).into(),
        surface: rgb(0x141e30).into(),
        surface_alt: rgb(0x192438).into(),
        surface_hover: rgb(0x233046).into(),
        border: rgb(0x283750).into(),
        divider: rgba(0x94a3b819).into(),
        text_primary: rgb(0xe2e8f0).into(),
        text_secondary: rgb(0x94a3b8).into(),
        text_muted: rgb(0x64748b).into(),
        accent: rgb(0x60a5fa).into(),
        accent_dim: rgba(0x60a5fa19).into(),
        accent_glow: rgba(0x60a5fa32).into(),
        success: rgb(0x4ade80).into(),
        success_dim: rgba(0x4ade8014).into(),
        warning: rgb(0xfbbf24).into(),
        warning_dim: rgba(0xfbbf2419).into(),
        danger: rgb(0xf87171).into(),
        danger_dim: rgba(0xf8717114).into(),
        sidebar_bg: rgb(0x060c16).into(),
        sidebar_text: rgb(0xcbd5e1).into(),
        sidebar_text_muted: rgb(0x64748b).into(),
        sidebar_active_bg: rgba(0x60a5fa23).into(),
        input_bg: rgb(0x0f1826).into(),
    }
}

// Spacing scale
pub const SPACE_XXS: f32 = 2.0;
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const SPACE_XXL: f32 = 32.0;

// Corner radii (in px)
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 10.0;
pub const RADIUS_XL: f32 = 14.0;

// Font sizes
pub const FONT_XS: f32 = 11.0;
pub const FONT_SM: f32 = 12.0;
pub const FONT_MD: f32 = 14.0;
pub const FONT_LG: f32 = 18.0;
pub const FONT_XL: f32 = 24.0;
pub const FONT_XXL: f32 = 32.0;

// Font families. These names match the OpenType `name` table family values
// of the bundled ttf files in `../assets/fonts/`. They are wired up by
// [`crate::fonts::init_fonts`] at startup so referencing them from any
// `.font_family(...)` call resolves the embedded glyphs instead of falling
// back to the system UI font (which is unreliable on macOS 26).
pub const FONT_SANS_FAMILY: &str = "Geist";
pub const FONT_MONO_FAMILY: &str = "Geist Mono";
