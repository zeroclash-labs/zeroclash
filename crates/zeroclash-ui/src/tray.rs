use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver};

use tray_icon::menu::{Menu, MenuEvent, MenuItem, MenuItemBuilder};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    Quit,
    SwitchMode(String),
    ToggleProxy,
    ToggleTun,
}

pub struct TrayManager {
    pub visible: Arc<AtomicBool>,
    event_rx: Receiver<TrayEvent>,
    _tray: TrayIcon,
}

fn menu_item(text: &str, id: &str) -> MenuItem {
    MenuItemBuilder::new()
        .text(text)
        .enabled(true)
        .id(id.into())
        .build()
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let visible = Arc::new(AtomicBool::new(true));

        let mut rgba = Vec::with_capacity(32 * 32 * 4);
        for _ in 0..(32 * 32) {
            rgba.extend_from_slice(&[66, 133, 244, 255]);
        }
        let icon = tray_icon::Icon::from_rgba(rgba, 32, 32)?;

        let menu = Menu::new();
        menu.append(&menu_item("Rule", "mode_rule"))?;
        menu.append(&menu_item("Global", "mode_global"))?;
        menu.append(&menu_item("Direct", "mode_direct"))?;
        menu.append(&MenuItem::new("---", false, None))?;
        menu.append(&menu_item("Toggle System Proxy", "proxy"))?;
        menu.append(&menu_item("Toggle TUN", "tun"))?;
        menu.append(&MenuItem::new("---", false, None))?;
        menu.append(&menu_item("Show/Hide", "show"))?;
        menu.append(&menu_item("Quit", "quit"))?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("ZeroClash")
            .with_icon(icon)
            .build()?;

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            loop {
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    let evt = match event.id().as_ref() {
                        "mode_rule" => TrayEvent::SwitchMode("rule".into()),
                        "mode_global" => TrayEvent::SwitchMode("global".into()),
                        "mode_direct" => TrayEvent::SwitchMode("direct".into()),
                        "proxy" => TrayEvent::ToggleProxy,
                        "tun" => TrayEvent::ToggleTun,
                        "quit" => TrayEvent::Quit,
                        _ => TrayEvent::ShowWindow,
                    };
                    let _ = tx.send(evt);
                }
                if TrayIconEvent::receiver().try_recv().is_ok() {
                    let _ = tx.send(TrayEvent::ShowWindow);
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });

        Ok(Self {
            visible,
            event_rx: rx,
            _tray: tray,
        })
    }

    pub fn poll(&self) -> Option<TrayEvent> {
        self.event_rx.try_recv().ok()
    }
}
