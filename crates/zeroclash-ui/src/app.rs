#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::hotkey::HotkeyManager;
use crate::state::AppState;
use crate::theme::init_theme;
use crate::tray::TrayManager;
use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;
use zeroclash_core::paths;

/// Launch the ZeroClash application.
pub fn run() {
    let log_dir = paths::log_dir();
    zeroclash_logging::init_logger(&log_dir).ok();

    log::info!("[zeroclash] run() entered");

    let _runtime_guard = crate::runtime::init();
    log::info!("[zeroclash] runtime::init OK");

    let data_dir = paths::data_dir();
    std::fs::create_dir_all(&data_dir).ok();
    log::info!("[zeroclash] data_dir created");

    zeroclash_i18n::sync_locale(None);
    log::info!("[zeroclash] sync_locale done");

    let tray = TrayManager::new().ok();
    log::info!("[zeroclash] TrayManager::new -> Some={}", tray.is_some());

    let hotkey = HotkeyManager::new();
    log::info!("[zeroclash] HotkeyManager::new done");

    log::info!("[zeroclash] about to call application().run(...)");
    application().run(move |cx: &mut App| {
        log::info!("[zeroclash] application().run closure entered");
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
