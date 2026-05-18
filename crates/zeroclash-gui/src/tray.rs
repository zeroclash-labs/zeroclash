//! System tray icon and menu via tray-icon crate.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    ToggleWindow,
    Quit,
}

pub struct SystemTray {
    pub visible: Arc<AtomicBool>,
    _tray: TrayIcon,
    _menu: Menu,
}

impl SystemTray {
    /// Create a system tray with menu items. Events are polled via `poll_events()`.
    pub fn new(
        visible: Arc<AtomicBool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 32x32 blue icon
        let mut rgba = Vec::with_capacity(32 * 32 * 4);
        for _ in 0..(32 * 32) {
            rgba.extend_from_slice(&[66, 133, 244, 255]);
        }
        let icon = tray_icon::Icon::from_rgba(rgba, 32, 32)?;

        let menu = Menu::new();
        let _show_item = MenuItem::new("Show/Hide", true, None);
        let _quit_item = MenuItem::new("Quit", true, None);
        menu.append(&_show_item)?;
        menu.append(&_quit_item)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("ZeroClash")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            visible,
            _tray: tray,
            _menu: menu,
        })
    }

    /// Poll for pending tray and menu events. Returns any action to take.
    pub fn poll_events() -> Option<TrayEvent> {
        // Check menu events
        if let Ok(_event) = MenuEvent::receiver().try_recv() {
            return Some(TrayEvent::ToggleWindow);
        }

        // Check tray icon click events
        if TrayIconEvent::receiver().try_recv().is_ok() {
            return Some(TrayEvent::ShowWindow);
        }

        None
    }
}
