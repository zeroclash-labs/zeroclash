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
    let log = |msg: &str| {
        if let Some(home) = dirs_next::home_dir() {
            let path = home.join("zeroclash-startup.log");
            let prev = std::fs::read_to_string(&path).unwrap_or_default();
            let _ = std::fs::write(&path, format!("{prev}{msg}\n"));
        }
    };
    log("[zeroclash] run() entered");

    let _runtime_guard = crate::runtime::init();
    log("[zeroclash] runtime::init OK");

    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zeroclash");
    std::fs::create_dir_all(&data_dir).ok();
    log("[zeroclash] data_dir created");

    zeroclash_i18n::sync_locale(None);
    log("[zeroclash] sync_locale done");

    let tray = TrayManager::new().ok();
    log(&format!(
        "[zeroclash] TrayManager::new -> Some={}",
        tray.is_some()
    ));

    let hotkey = HotkeyManager::new();
    log("[zeroclash] HotkeyManager::new done");

    log("[zeroclash] about to call application().run(...)");
    application().run(move |cx: &mut App| {
        if let Some(home) = dirs_next::home_dir() {
            let path = home.join("zeroclash-startup.log");
            let prev = std::fs::read_to_string(&path).unwrap_or_default();
            let _ = std::fs::write(
                &path,
                format!("{prev}[zeroclash] application().run closure entered\n"),
            );
        }
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
