//! Embedded font registration.
//!
//! GPUI 1.3.7's default `font_family` is `.SystemUIFont`, which on macOS 26
//! (Darwin 25) silently fails to resolve via font-kit — `font_kit::source::Source::select_family_by_name`
//! returns `Err(NoSuchFamily)` for both `.AppleSystemUIFont` and most
//! bundled fallbacks (`Helvetica` works, almost nothing else does).
//!
//! On top of that, GPUI's own [`PlatformTextSystem::add_fonts`] implementation
//! pipes embedded ttf data through `CGFont::from_data_provider` →
//! `font_kit::loaders::core_text::Font::from_core_graphics_font` →
//! `font_kit::sources::mem::MemSource::add_fonts`. On macOS 26 that path
//! accepts the bytes (returns `Ok`) but the resulting fonts never appear in
//! `all_font_names()` — the family-name indexing inside font-kit's
//! `MemSource` doesn't pick them up.
//!
//! Workaround: register the bundled ttf files directly with CoreText via
//! `CTFontManagerRegisterGraphicsFont`. As a safety net, if the fonts still
//! don't appear in `all_font_names()` after registration, we fall back to
//! "Helvetica" — the one family font-kit reliably resolves on macOS 26.

use std::sync::OnceLock;

use gpui::App;

const GEIST_REGULAR: &[u8] = include_bytes!("../assets/fonts/Geist-Regular.ttf");
const GEIST_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Geist-Medium.ttf");
const GEIST_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Geist-SemiBold.ttf");
const GEIST_BOLD: &[u8] = include_bytes!("../assets/fonts/Geist-Bold.ttf");
const GEIST_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/GeistMono-Regular.ttf");

const EMBEDDED: &[(&str, &[u8])] = &[
    ("Geist-Regular", GEIST_REGULAR),
    ("Geist-Medium", GEIST_MEDIUM),
    ("Geist-SemiBold", GEIST_SEMIBOLD),
    ("Geist-Bold", GEIST_BOLD),
    ("GeistMono-Regular", GEIST_MONO_REGULAR),
];

static SANS_FAMILY: OnceLock<String> = OnceLock::new();
static MONO_FAMILY: OnceLock<String> = OnceLock::new();

/// Best available sans-serif family after font registration.
pub fn sans_family() -> &'static str {
    SANS_FAMILY.get().map(|s| s.as_str()).unwrap_or("Geist")
}

/// Best available monospace family after font registration.
pub fn mono_family() -> &'static str {
    MONO_FAMILY
        .get()
        .map(|s| s.as_str())
        .unwrap_or("Geist Mono")
}

/// Register Geist Sans + Geist Mono with the system font collection.
///
/// Must be called inside the `Application::run` closure before any window
/// is opened so that the first frame can resolve the family names.
pub fn init_fonts(cx: &App) {
    #[cfg(target_os = "macos")]
    let counts = macos::register_with_core_text();

    #[cfg(not(target_os = "macos"))]
    let counts = register_via_gpui(cx);

    let names = cx.text_system().all_font_names();
    let has_geist = names.iter().any(|n| n == "Geist");
    let has_geist_mono = names.iter().any(|n| n == "Geist Mono");

    let sans = if has_geist { "Geist" } else { "Helvetica" };
    let mono = if has_geist_mono {
        "Geist Mono"
    } else {
        "Helvetica"
    };

    // Unwrap safety: init_fonts is called once, so set() always succeeds.
    SANS_FAMILY.set(sans.to_string()).unwrap();
    MONO_FAMILY.set(mono.to_string()).unwrap();

    let diag = format!(
        "[zc-fonts] registered {}/{} ttf, \
         families total={} Geist={has_geist} GeistMono={has_geist_mono} \
         -> sans={sans} mono={mono}\n",
        counts.0,
        counts.1,
        names.len(),
    );
    log::info!("{diag}");
}

#[cfg(not(target_os = "macos"))]
fn register_via_gpui(cx: &App) -> (usize, usize) {
    use std::borrow::Cow;
    let fonts: Vec<Cow<'static, [u8]>> = EMBEDDED
        .iter()
        .map(|(_, bytes)| Cow::Borrowed(*bytes))
        .collect();
    let total = fonts.len();
    match cx.text_system().add_fonts(fonts) {
        Ok(()) => (total, total),
        Err(err) => {
            log::error!("add_fonts failed: {err:?}");
            (0, total)
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use core_graphics::data_provider::CGDataProvider;
    use core_graphics::font::CGFont;
    use core_graphics::sys::CGFontRef;
    use foreign_types_shared::ForeignType;

    use super::EMBEDDED;

    #[link(name = "CoreText", kind = "framework")]
    unsafe extern "C" {
        fn CTFontManagerRegisterGraphicsFont(
            font: CGFontRef,
            error: *mut *mut std::ffi::c_void,
        ) -> bool;
    }

    pub fn register_with_core_text() -> (usize, usize) {
        let mut ok = 0usize;
        for (name, bytes) in EMBEDDED {
            let provider = unsafe { CGDataProvider::from_slice(bytes) };
            let Ok(font) = CGFont::from_data_provider(provider) else {
                log::warn!("CGFont::from_data_provider failed for {name}");
                continue;
            };
            let mut err_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let success = unsafe { CTFontManagerRegisterGraphicsFont(font.as_ptr(), &mut err_ptr) };
            if success {
                ok += 1;
            } else if !err_ptr.is_null() {
                log::warn!("CTFontManagerRegisterGraphicsFont failed for {name}");
            }
        }
        (ok, EMBEDDED.len())
    }
}
