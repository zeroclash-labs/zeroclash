//! Internationalization integration. Wraps zeroclash-i18n.

use std::borrow::Cow;

/// Translate a key. Returns owned key if no translation found.
#[inline]
pub fn tr(key: &str) -> Cow<'static, str> {
    let translated = zeroclash_i18n::translate(key);
    if translated.as_ref() == key {
        Cow::Owned(key.to_string())
    } else {
        Cow::Owned(translated.into_owned())
    }
}

#[inline]
pub fn set_language(lang: &str) {
    zeroclash_i18n::set_locale(lang);
}

#[inline]
pub fn system_language() -> Cow<'static, str> {
    zeroclash_i18n::system_language()
}
