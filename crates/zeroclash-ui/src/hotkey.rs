//! Global hotkey integration.
//!
//! The previous implementation:
//!
//! 1. Constructed `GlobalHotKeyManager` inside an `if let Ok(manager) = ...`
//!    block but never stored the manager anywhere, so the hotkeys were
//!    immediately unregistered when the block ended.
//! 2. Spawned **two** background threads, both calling
//!    `GlobalHotKeyEvent::receiver().try_recv()` on the same global event
//!    channel. Whichever thread happened to win the race consumed the event
//!    and the other action was randomly lost.
//!
//! This module fixes both: the manager is owned by `HotkeyManager` for the
//! whole process lifetime, and a single poll thread maps incoming event ids
//! to the right [`HotkeyAction`] before forwarding them to the UI thread.

use std::sync::mpsc::{self, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleProxy,
    ShowWindow,
}

pub struct HotkeyManager {
    event_rx: Receiver<HotkeyAction>,
    // Held purely for its `Drop` side effect — dropping the manager
    // unregisters every hotkey it owns. We never read this field on
    // non-macOS builds, hence the cfg gate.
    #[cfg(target_os = "macos")]
    _manager: Option<global_hotkey::GlobalHotKeyManager>,
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
        let manager = Self::register_macos(&tx);

        #[cfg(not(target_os = "macos"))]
        {
            let _ = tx;
            log::info!("Global hotkeys are only supported on macOS in this build");
        }

        Self {
            event_rx: rx,
            #[cfg(target_os = "macos")]
            _manager: manager,
        }
    }

    pub fn poll(&self) -> Option<HotkeyAction> {
        self.event_rx.try_recv().ok()
    }

    #[cfg(target_os = "macos")]
    fn register_macos(
        tx: &std::sync::mpsc::Sender<HotkeyAction>,
    ) -> Option<global_hotkey::GlobalHotKeyManager> {
        use global_hotkey::GlobalHotKeyEvent;
        use global_hotkey::hotkey::{Code, HotKey, Modifiers};

        let manager = match global_hotkey::GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("failed to create global hotkey manager: {e}");
                return None;
            }
        };

        let toggle_proxy = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyP);
        let show_window = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyO);

        let mut bindings: Vec<(u32, HotkeyAction)> = Vec::new();
        match manager.register(toggle_proxy) {
            Ok(()) => bindings.push((toggle_proxy.id, HotkeyAction::ToggleProxy)),
            Err(e) => log::warn!("failed to register Cmd+Shift+P (toggle proxy): {e}"),
        }
        match manager.register(show_window) {
            Ok(()) => bindings.push((show_window.id, HotkeyAction::ShowWindow)),
            Err(e) => log::warn!("failed to register Cmd+Shift+O (show window): {e}"),
        }

        if bindings.is_empty() {
            return Some(manager);
        }

        let tx = tx.clone();
        std::thread::Builder::new()
            .name("zeroclash-hotkey-poll".into())
            .spawn(move || {
                let receiver = GlobalHotKeyEvent::receiver();
                loop {
                    while let Ok(event) = receiver.try_recv() {
                        if let Some((_id, action)) = bindings.iter().find(|(id, _)| *id == event.id)
                            && tx.send(*action).is_err()
                        {
                            return;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
            .ok();

        Some(manager)
    }
}
