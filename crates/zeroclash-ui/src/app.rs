#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use crate::hotkey::HotkeyManager;
use crate::state::AppState;
use crate::theme::init_theme;
use crate::tray::TrayManager;
use gpui::{prelude::*, px, size, App, Bounds, WindowBounds, WindowOptions};
use gpui_platform::application;

/// Launch the ZeroClash application.
pub fn run() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let _guard = rt.enter();

    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zeroclash");
    std::fs::create_dir_all(&data_dir).ok();

    let tray = TrayManager::new().ok();
    let hotkey = HotkeyManager::new();

    application().run(move |cx: &mut App| {
        init_theme(cx, "");

        let bounds = Bounds::centered(None, size(px(1280.0), px(840.0)), cx);
        let dd = data_dir.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_window, cx| cx.new(|_cx| AppState::new(dd.clone(), tray, hotkey)),
        )
        .unwrap();

        cx.activate(true);
    });
}
