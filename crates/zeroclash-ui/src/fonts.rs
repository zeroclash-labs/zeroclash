//! Embedded font registration.
//!
//! GPUI 1.3.7's default `font_family` is `.SystemUIFont`, which on macOS 26
//! (Darwin 25) and recent Linux distributions can fail to resolve via
//! font-kit / CoreText, leading to silent text rendering failures (the
//! window opens but no glyphs are drawn). Registering our own bundled
//! fonts and explicitly setting the root `font_family` sidesteps that path
//! entirely and gives us a consistent visual identity across platforms.

use std::borrow::Cow;

use gpui::App;

const GEIST_REGULAR: &[u8] = include_bytes!("../assets/fonts/Geist-Regular.ttf");
const GEIST_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Geist-Medium.ttf");
const GEIST_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Geist-SemiBold.ttf");
const GEIST_BOLD: &[u8] = include_bytes!("../assets/fonts/Geist-Bold.ttf");
const GEIST_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/GeistMono-Regular.ttf");

/// Register Geist Sans + Geist Mono with the GPUI text system.
///
/// Must be called inside the `Application::run` closure before any window
/// is opened so that the first frame can resolve `Geist` / `Geist Mono`
/// family names.
pub fn init_fonts(cx: &App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(GEIST_REGULAR),
        Cow::Borrowed(GEIST_MEDIUM),
        Cow::Borrowed(GEIST_SEMIBOLD),
        Cow::Borrowed(GEIST_BOLD),
        Cow::Borrowed(GEIST_MONO_REGULAR),
    ];

    if let Err(err) = cx.text_system().add_fonts(fonts) {
        log::error!(
            "failed to register embedded Geist fonts; falling back to system UI font: {err:?}"
        );
    }
}
