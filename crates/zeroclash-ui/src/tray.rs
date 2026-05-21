use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver};

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    Quit,
}

pub struct TrayManager {
    pub visible: Arc<AtomicBool>,
    event_rx: Receiver<TrayEvent>,
    _tray: TrayIcon,
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
        let show_item = MenuItem::new("Show/Hide", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&show_item)?;
        menu.append(&quit_item)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("ZeroClash")
            .with_icon(icon)
            .build()?;

        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            loop {
                if MenuEvent::receiver().try_recv().is_ok() {
                    let _ = tx.send(TrayEvent::ShowWindow);
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
