use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone)]
pub enum HotkeyAction {
    ToggleProxy,
    ShowWindow,
}

pub struct HotkeyManager {
    event_rx: Receiver<HotkeyAction>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        #[cfg(target_os = "macos")]
        {
            if let Ok(manager) = global_hotkey::GlobalHotKeyManager::new() {
                use global_hotkey::GlobalHotKeyEvent;
                use global_hotkey::hotkey::{Code, HotKey, Modifiers};

                let hk = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyP);
                let hk_id = hk.id;
                if manager.register(hk).is_ok() {
                    let tx = tx.clone();
                    std::thread::spawn(move || {
                        loop {
                            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv()
                                && event.id == hk_id
                            {
                                let _ = tx.send(HotkeyAction::ToggleProxy);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    });
                }

                let hk2 = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyO);
                let hk2_id = hk2.id;
                if manager.register(hk2).is_ok() {
                    let _tx = tx;
                    std::thread::spawn(move || {
                        loop {
                            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv()
                                && event.id == hk2_id
                            {
                                let _ = _tx.send(HotkeyAction::ShowWindow);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                    });
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = tx;
            log::info!("Global hotkeys only supported on macOS");
        }

        Self { event_rx: rx }
    }

    pub fn poll(&self) -> Option<HotkeyAction> {
        self.event_rx.try_recv().ok()
    }
}
