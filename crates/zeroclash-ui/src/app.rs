#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use crate::hotkey::HotkeyManager;
use crate::state::AppState;
use crate::theme::init_theme;
use crate::tray::TrayManager;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

/// Launch the ZeroClash application.
pub fn run() {
    // The guard binds the leaked tokio runtime to this (main) thread for the
    // entire `application().run(...)` call, so any `pollster::block_on` of a
    // `tokio::*` future during rendering can find the runtime via
    // `Handle::current()`. The leaked runtime itself lives forever, so any
    // tasks spawned via `crate::runtime::handle()` from other threads are
    // also safe.
    let _runtime_guard = crate::runtime::init();

    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zeroclash");
    std::fs::create_dir_all(&data_dir).ok();

    // Resolve initial locale: fall back to system locale if no value has
    // been persisted in `clash-verge.yaml`. Subsequent `Language` toggles
    // in Settings call `zeroclash_i18n::set_locale` directly.
    zeroclash_i18n::sync_locale(None);

    let tray = TrayManager::new().ok();
    let hotkey = HotkeyManager::new();

    application().run(move |cx: &mut App| {
        crate::fonts::init_fonts(cx);
        init_theme(cx, "");

        let bounds = Bounds::centered(None, size(px(1280.0), px(840.0)), cx);
        let dd = data_dir.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_window, cx| cx.new(|cx| AppState::new(cx, dd.clone(), tray, hotkey)),
        )
        .unwrap();

        cx.activate(true);
    });
}
