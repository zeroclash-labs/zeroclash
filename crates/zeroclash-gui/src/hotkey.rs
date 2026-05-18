//! Global hotkey registration for system-wide keyboard shortcuts.

use std::sync::mpsc;
use winit::event_loop::EventLoopProxy;

/// Actions that can be triggered by global hotkeys.
#[derive(Debug, Clone)]
pub enum HotkeyAction {
    ToggleProxy,
    ToggleTun,
    ShowWindow,
}

/// Manages global hotkey registration.
pub struct HotkeyManager {
    _manager: Option<global_hotkey::GlobalHotKeyManager>,
}

impl HotkeyManager {
    /// Create and register global hotkeys.
    /// Returns a receiver for hotkey events and the manager handle.
    pub fn register(
        proxy: EventLoopProxy<HotkeyAction>,
    ) -> Result<(Self, mpsc::Receiver<HotkeyAction>), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel::<HotkeyAction>();

        let manager = match global_hotkey::GlobalHotKeyManager::new() {
            Ok(m) => Some(m),
            Err(e) => {
                log::warn!("Global hotkey manager unavailable: {e}");
                None
            }
        };

        #[cfg(target_os = "macos")]
        if let Some(ref mgr) = manager {
            // Cmd+Shift+P: toggle proxy
            use global_hotkey::hotkey::{HotKey, Modifiers};
            use global_hotkey::GlobalHotKeyEvent;

            let hk = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), global_hotkey::hotkey::Code::KeyP);
            let hk_id = hk.id;
            if mgr.register(hk).is_ok() {
                let tx_clone = tx.clone();
                std::thread::spawn(move || {
                    let receiver = GlobalHotKeyEvent::receiver();
                    loop {
                        if let Ok(event) = receiver.try_recv() {
                            if event.id == hk_id {
                                let _ = tx_clone.send(HotkeyAction::ToggleProxy);
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                });
            }

            // Cmd+Shift+O: show window
            let hk2 = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), global_hotkey::hotkey::Code::KeyO);
            let hk2_id = hk2.id;
            if mgr.register(hk2).is_ok() {
                let tx_clone2 = tx.clone();
                std::thread::spawn(move || {
                    let receiver = GlobalHotKeyEvent::receiver();
                    loop {
                        if let Ok(event) = receiver.try_recv() {
                            if event.id == hk2_id {
                                let _ = tx_clone2.send(HotkeyAction::ShowWindow);
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                });
            }

            let _ = proxy; // EventLoopProxy for future winit event integration
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = proxy;
            let _ = tx;
            log::info!("Global hotkeys only supported on macOS for now");
        }

        Ok((Self { _manager: manager }, rx))
    }
}
