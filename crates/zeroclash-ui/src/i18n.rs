//! UI-side translation helper.
//!
//! Wraps [`zeroclash_i18n::translate`] so call sites can hand the result
//! directly to GPUI element APIs that expect a [`SharedString`]. Keeps
//! views terse:
//!
//! ```ignore
//! .child(tr("ui.nav.dashboard"))
//! ```

use gpui::SharedString;

/// Translate `key` using the active locale and return a `SharedString`
/// suitable for `.child(...)` calls.
#[inline]
pub fn tr(key: &str) -> SharedString {
    SharedString::from(zeroclash_i18n::translate(key).into_owned())
}

/// Like [`tr`] but applies a single `{name}` placeholder substitution.
#[inline]
pub fn tr_arg(key: &str, name: &str, value: &str) -> SharedString {
    let raw = zeroclash_i18n::translate(key).into_owned();
    let placeholder = format!("{{{name}}}");
    SharedString::from(raw.replace(&placeholder, value))
}
