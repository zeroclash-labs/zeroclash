#![allow(clippy::expect_used, clippy::unwrap_used)]

use gpui::{App, AppContext as _, Bounds, WindowBounds, WindowOptions, px, size};

use crate::hotkey::HotkeyManager;
use crate::state::AppState;
use crate::theme::init_theme;
use crate::tray::TrayManager;

/// Launch the ZeroClash application.
pub fn run() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let _guard = rt.enter();

    let tray = TrayManager::new().ok();
    let hotkey = HotkeyManager::new();

    gpui_platform::application().run(|cx: &mut App| {
        init_theme(cx, "");

        let bounds = Bounds::centered(None, size(px(1280.0), px(840.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| AppState::new(tray, hotkey)),
        )
        .unwrap();

        cx.activate(true);
    });
}
