use gpui::{
    Context, CursorStyle, FocusHandle, KeyDownEvent, MouseButton, Render, Window, div, prelude::*,
    px, white,
};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use zeroclash_core::CoreManager;
use zeroclash_core::MihomoClient;
use zeroclash_core::config::VergeConfig;
use zeroclash_core::mihomo::ProxyGroup;
use zeroclash_core::profile::{ProfilePreview, ProfileStore};
use zeroclash_core::{Config, SystemProxy, notify};

use crate::components::log_viewer::{LogLevel, LogViewer};
use crate::components::traffic_graph::TrafficHistory;
use crate::design::{self, Colors, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS};
use crate::hotkey::HotkeyManager;
use crate::i18n::tr;
use crate::theme::Theme;
use crate::tray::TrayManager;
use crate::util::{self, CachedConn};
use crate::views::{connections, dashboard, logs, profiles, proxies, settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Home,
    Proxies,
    Profiles,
    Connections,
    Logs,
    Settings,
}

pub enum UiCommand {
    Navigate(Page),
    ToggleCore,
    ToggleSystemProxy,
    ToggleAutoStart,
    RefreshProxies,
    RefreshConnections,
    ImportProfile(String),
    ActivateProfile(String),
    DeleteProfile(String),
    CloseConnection(String),
    SelectProxy(String, String),
    SwitchMode(String),
    ToggleTun,
    TestDelay(String),
}

/// Events posted by background tokio tasks back to the UI thread.
///
/// `process_commands` spawns short-lived tasks for any IO that we don't
/// want to block the GPUI render loop on (mihomo REST polling, proxy
/// switching, latency probes, etc.). Those tasks send their results as
/// `UiEvent` variants over an unbounded channel which `AppState` drains
/// every frame from `drain_events`.
pub enum UiEvent {
    ProxiesRefreshed(Vec<ProxyGroup>),
    ConnectionsRefreshed(Vec<CachedConn>),
    DelayMeasured {
        name: String,
        delay_ms: u64,
    },
    Log {
        level: LogLevel,
        module: String,
        message: String,
    },
    Notify {
        title: String,
        body: String,
    },
    /// Re-issue another `UiCommand` once a prior async task has finished
    /// (e.g. refresh proxies after `select_proxy` succeeded).
    PostCommand(UiCommand),
    /// Sent by the supervisor task when the running mihomo core has been
    /// unreachable for several consecutive probes — likely crashed.
    CoreCrashed,
}

pub struct AppState {
    pub current_page: Page,
    pub core_running: bool,
    pub traffic: TrafficHistory,
    pub proxy_groups: Vec<ProxyGroup>,
    pub connections: Vec<CachedConn>,
    pub enable_system_proxy: bool,
    pub profile_previews: Vec<ProfilePreview>,
    pub import_dialog_visible: bool,
    pub import_url: String,
    pub import_url_error: Option<String>,
    /// When `Some(uid)`, the profiles page row for `uid` is showing an
    /// inline "delete confirmation" prompt instead of the regular
    /// Activate / Delete buttons.
    pub pending_delete_uid: Option<String>,
    pub selected_conn_id: Option<String>,
    pub log_viewer: LogViewer,
    pub data_dir: PathBuf,
    pub config: Config,
    pending_commands: Vec<UiCommand>,
    tray: Option<TrayManager>,
    hotkey: HotkeyManager,
    core_manager: Option<CoreManager>,
    profile_store: Option<ProfileStore>,
    client: Option<MihomoClient>,
    frame_count: u64,
    event_tx: UnboundedSender<UiEvent>,
    event_rx: UnboundedReceiver<UiEvent>,
    /// Set when a tray click or hotkey requests focus. Consumed by
    /// `Render::render` which calls `window.activate_window()` on the
    /// next frame.
    want_activate: bool,
    /// Background task that periodically pings the running core and
    /// posts `UiEvent::CoreCrashed` after 3 consecutive failures.
    core_supervisor: Option<tokio::task::JoinHandle<()>>,
    /// Focus handle for the profile import dialog's URL input. Allocated
    /// once at construction so the dialog can receive `KeyDownEvent`s
    /// when shown.
    pub import_focus: FocusHandle,
    /// Focus handle for the logs page's "type-to-filter" search field.
    pub logs_search_focus: FocusHandle,
    /// Focus + active query for the proxies page's node filter.
    pub proxies_search_focus: FocusHandle,
    pub proxies_filter: String,
    pub proxies_filter_lower: String,
    /// Focus handle for the sidebar — used to receive arrow-key
    /// navigation between pages.
    pub sidebar_focus: FocusHandle,
}

impl AppState {
    pub fn new(
        cx: &mut Context<Self>,
        data_dir: PathBuf,
        tray: Option<TrayManager>,
        hotkey: HotkeyManager,
    ) -> Self {
        let mut lv = LogViewer::default();
        lv.store
            .push(LogLevel::Info, "zeroclash", "Application started");

        let config = Self::load_config(&data_dir);
        let enable_system_proxy = config.verge.latest_arc().enable_system_proxy;
        let profile_store = pollster::block_on(ProfileStore::load(data_dir.clone())).ok();
        let profile_previews = profile_store
            .as_ref()
            .map(|ps| ps.preview())
            .unwrap_or_default();

        let (event_tx, event_rx) = unbounded_channel();

        Self {
            current_page: Page::Home,
            core_running: false,
            traffic: TrafficHistory::default(),
            proxy_groups: Vec::new(),
            connections: Vec::new(),
            enable_system_proxy,
            profile_previews,
            import_dialog_visible: false,
            import_url: String::new(),
            import_url_error: None,
            pending_delete_uid: None,
            selected_conn_id: None,
            log_viewer: lv,
            data_dir,
            config,
            pending_commands: Vec::new(),
            tray,
            hotkey,
            core_manager: None,
            profile_store,
            client: None,
            frame_count: 0,
            event_tx,
            event_rx,
            want_activate: false,
            core_supervisor: None,
            import_focus: cx.focus_handle(),
            logs_search_focus: cx.focus_handle(),
            proxies_search_focus: cx.focus_handle(),
            proxies_filter: String::new(),
            proxies_filter_lower: String::new(),
            sidebar_focus: cx.focus_handle(),
        }
    }

    /// Cycle the active page in the sidebar by +/- 1.
    pub fn cycle_page(&mut self, delta: i32) {
        let pages = [
            Page::Home,
            Page::Proxies,
            Page::Profiles,
            Page::Connections,
            Page::Logs,
            Page::Settings,
        ];
        let len = pages.len() as i32;
        let current = pages
            .iter()
            .position(|p| *p == self.current_page)
            .unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len) as usize;
        self.current_page = pages[next].clone();
    }

    /// Update the proxies page filter, recomputing the lowercase form.
    pub fn set_proxies_filter(&mut self, query: String) {
        self.proxies_filter_lower = query.to_lowercase();
        self.proxies_filter = query;
    }

    /// Drain background task results into UI state. Called every frame
    /// before `process_commands` so freshly arrived data feeds the next
    /// tick of command handling.
    fn drain_events(&mut self) {
        while let Ok(ev) = self.event_rx.try_recv() {
            match ev {
                UiEvent::ProxiesRefreshed(groups) => self.proxy_groups = groups,
                UiEvent::ConnectionsRefreshed(conns) => {
                    let valid: HashSet<&str> = conns.iter().map(|c| c.entry.id.as_str()).collect();
                    if let Some(id) = self.selected_conn_id.as_deref()
                        && !valid.contains(id)
                    {
                        self.selected_conn_id = None;
                    }
                    self.connections = conns;
                }
                UiEvent::DelayMeasured { name, delay_ms } => {
                    self.log_viewer.store.push(
                        LogLevel::Info,
                        "proxy",
                        &format!("{name}: {delay_ms}ms"),
                    );
                    self.push_command(UiCommand::RefreshProxies);
                }
                UiEvent::Log {
                    level,
                    module,
                    message,
                } => {
                    self.log_viewer.store.push(level, &module, &message);
                }
                UiEvent::Notify { title, body } => notify(&title, &body),
                UiEvent::PostCommand(cmd) => self.push_command(cmd),
                UiEvent::CoreCrashed => self.handle_core_crashed(),
            }
        }
    }

    fn handle_core_crashed(&mut self) {
        if !self.core_running {
            return;
        }
        if let Some(handle) = self.core_supervisor.take() {
            handle.abort();
        }
        self.core_running = false;
        self.client = None;
        self.core_manager = None;
        self.proxy_groups.clear();
        self.connections.clear();
        self.log_viewer.store.push(
            LogLevel::Error,
            "core",
            "Core appears to have stopped unexpectedly",
        );
        notify("ZeroClash", "Core stopped unexpectedly");
    }

    fn spawn_supervise_core(&self) -> Option<tokio::task::JoinHandle<()>> {
        let client = self.client.clone()?;
        let tx = self.event_tx.clone();
        Some(crate::runtime::handle().spawn(async move {
            let mut consecutive_failures: u32 = 0;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                if client.version().await.is_ok() {
                    consecutive_failures = 0;
                } else {
                    consecutive_failures += 1;
                    if consecutive_failures >= 3 {
                        let _ = tx.send(UiEvent::CoreCrashed);
                        break;
                    }
                }
            }
        }))
    }

    fn load_config(data_dir: &std::path::Path) -> Config {
        let path = data_dir.join("clash-verge.yaml");
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_yaml_ng::from_str::<VergeConfig>(&content) {
                    Ok(verge) => Config::from_verge(verge),
                    Err(e) => {
                        log::warn!("Failed to parse clash-verge.yaml: {e}, using defaults");
                        Config::new()
                    }
                },
                Err(_) => Config::new(),
            }
        } else {
            Config::new()
        }
    }

    pub fn save_config(&self) {
        let path = self.data_dir.join("clash-verge.yaml");
        let verge = self.config.verge.latest_arc();
        match serde_yaml_ng::to_string(&*verge) {
            Ok(yaml) => {
                if let Err(e) = std::fs::write(&path, yaml) {
                    log::error!("Failed to save config: {e}");
                }
            }
            Err(e) => log::error!("Failed to serialize config: {e}"),
        }
    }

    fn refresh_profiles(&mut self) {
        if let Some(ref ps) = self.profile_store {
            self.profile_previews = ps.preview();
        }
    }

    fn handle_import_profile(&mut self, url: String) {
        let Some(ref mut ps) = self.profile_store else {
            return;
        };
        // TODO(p1-render-blocking): split fetch_remote into a spawned IO
        // task; the file write + reqwest call still blocks the UI thread.
        match pollster::block_on(ps.fetch_remote(&url, None, None)) {
            Ok(item) => {
                if let Err(e) = ps.add_item(item) {
                    self.log_viewer.store.push(
                        LogLevel::Error,
                        "profile",
                        &format!("Failed to add: {e}"),
                    );
                }
                if let Err(e) = pollster::block_on(ps.save()) {
                    self.log_viewer.store.push(
                        LogLevel::Error,
                        "profile",
                        &format!("Failed to save: {e}"),
                    );
                }
                self.refresh_profiles();
                self.log_viewer
                    .store
                    .push(LogLevel::Info, "profile", &format!("Imported {url}"));
                notify("ZeroClash", "Profile imported");
            }
            Err(e) => {
                self.log_viewer.store.push(
                    LogLevel::Error,
                    "profile",
                    &format!("Failed to fetch: {e}"),
                );
            }
        }
    }

    fn handle_activate_profile(&mut self, uid: String) {
        let Some(ref mut ps) = self.profile_store else {
            return;
        };
        if let Err(e) = ps.set_current(&uid) {
            self.log_viewer.store.push(
                LogLevel::Error,
                "profile",
                &format!("Failed to activate: {e}"),
            );
            return;
        }
        let _ = pollster::block_on(ps.save());
        self.refresh_profiles();
        self.log_viewer
            .store
            .push(LogLevel::Info, "profile", &format!("Activated {uid}"));
        notify("ZeroClash", "Profile activated");
        self.spawn_apply_current_profile();
    }

    /// Resolve the absolute path of the currently active profile's YAML
    /// file. Returns `None` if no profile is active or the profile store
    /// is unavailable.
    fn current_profile_path(&self) -> Option<PathBuf> {
        let ps = self.profile_store.as_ref()?;
        let current = ps.profiles.current.as_deref()?;
        let items = ps.profiles.items.as_ref()?;
        let item = items.iter().find(|i| i.uid.as_deref() == Some(current))?;
        let file = item.file.as_deref()?;
        Some(self.data_dir.join(file))
    }

    /// Tell the running mihomo core to hot-reload the YAML file pointed
    /// to by the currently active profile via `PUT /configs`.
    fn spawn_apply_current_profile(&self) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(path) = self.current_profile_path() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            let payload = serde_json::json!({ "path": path.to_string_lossy() });
            match client.patch_config(&payload).await {
                Ok(()) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Info,
                        module: "core".into(),
                        message: format!("core reloaded with {}", path.display()),
                    });
                    let _ = tx.send(UiEvent::PostCommand(UiCommand::RefreshProxies));
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "core".into(),
                        message: format!("core reload failed: {e}"),
                    });
                }
            }
        });
    }

    /// Tell the running mihomo core to enable/disable TUN mode via
    /// `PUT /configs` with `{"tun": {"enable": <bool>}}`.
    fn spawn_apply_tun(&self) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let enabled = self.config.verge.latest_arc().enable_tun;
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            let payload = serde_json::json!({ "tun": { "enable": enabled } });
            match client.patch_config(&payload).await {
                Ok(()) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Info,
                        module: "core".into(),
                        message: format!(
                            "core TUN mode {}",
                            if enabled { "enabled" } else { "disabled" }
                        ),
                    });
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "core".into(),
                        message: format!("core TUN apply failed: {e}"),
                    });
                }
            }
        });
    }

    fn handle_delete_profile(&mut self, uid: String) {
        let Some(ref mut ps) = self.profile_store else {
            return;
        };
        match ps.delete_item(&uid) {
            Ok(_) => {
                let _ = pollster::block_on(ps.save());
                self.refresh_profiles();
                self.log_viewer
                    .store
                    .push(LogLevel::Info, "profile", &format!("Deleted {uid}"));
                notify("ZeroClash", "Profile deleted");
            }
            Err(e) => {
                self.log_viewer.store.push(
                    LogLevel::Error,
                    "profile",
                    &format!("Failed to delete: {e}"),
                );
            }
        }
    }

    fn handle_toggle_core(&mut self) {
        if self.core_running {
            if let Some(handle) = self.core_supervisor.take() {
                handle.abort();
            }
            // TODO: move stop into the background and surface the result
            // via UiEvent so the UI never freezes when the core hangs.
            if let Some(ref cm) = self.core_manager
                && let Err(e) = pollster::block_on(cm.stop())
            {
                self.log_viewer.store.push(
                    LogLevel::Error,
                    "core",
                    &format!("Failed to stop: {e}"),
                );
            }
            self.core_running = false;
            self.client = None;
            self.core_manager = None;
            self.proxy_groups.clear();
            self.connections.clear();
            self.log_viewer
                .store
                .push(LogLevel::Info, "core", "Core stopped");
            notify("ZeroClash", "Core stopped");
        } else {
            let core_path = {
                let verge = self.config.verge.latest_arc();
                if verge.clash_core_path.is_empty() {
                    "mihomo".to_string()
                } else {
                    verge.clash_core_path.clone()
                }
            };

            let cm = CoreManager::new(PathBuf::from(&core_path), None);

            match pollster::block_on(cm.start()) {
                Ok(()) => {
                    self.client = Some(cm.client());
                    self.core_manager = Some(cm);
                    self.core_running = true;
                    self.core_supervisor = self.spawn_supervise_core();
                    self.log_viewer.store.push(
                        LogLevel::Info,
                        "core",
                        &format!("Core started from {core_path}"),
                    );
                    notify("ZeroClash", "Core started");
                }
                Err(e) => {
                    self.log_viewer.store.push(
                        LogLevel::Error,
                        "core",
                        &format!("Failed to start: {e}"),
                    );
                }
            }
        }
    }

    fn handle_toggle_tun(&mut self) {
        self.config
            .verge
            .edit_draft(|v| v.enable_tun = !v.enable_tun);
        self.config.verge.apply();
        self.save_config();
        let enabled = self.config.verge.latest_arc().enable_tun;
        self.log_viewer.store.push(
            LogLevel::Info,
            "core",
            &format!("TUN mode {}", if enabled { "enabled" } else { "disabled" }),
        );
        notify(
            "ZeroClash",
            if enabled {
                "TUN mode enabled"
            } else {
                "TUN mode disabled"
            },
        );
        self.spawn_apply_tun();
    }

    fn spawn_refresh_proxies(&self) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            match client.proxies().await {
                Ok(v) => {
                    let _ = tx.send(UiEvent::ProxiesRefreshed(util::parse_proxy_groups(&v)));
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "proxy".into(),
                        message: format!("refresh proxies failed: {e}"),
                    });
                }
            }
        });
    }

    fn spawn_refresh_connections(&self) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            match client.connections().await {
                Ok(v) => {
                    let _ = tx.send(UiEvent::ConnectionsRefreshed(
                        util::parse_cached_connections(&v),
                    ));
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "conn".into(),
                        message: format!("refresh connections failed: {e}"),
                    });
                }
            }
        });
    }

    fn spawn_select_proxy(&self, group: String, proxy: String) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            match client.select_proxy(&group, &proxy).await {
                Ok(()) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Info,
                        module: "proxy".into(),
                        message: format!("Switched {group} -> {proxy}"),
                    });
                    let _ = tx.send(UiEvent::PostCommand(UiCommand::RefreshProxies));
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "proxy".into(),
                        message: format!("Switch failed: {e}"),
                    });
                }
            }
        });
    }

    fn spawn_switch_mode(&self, mode: String) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            match client.switch_mode(&mode).await {
                Ok(()) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Info,
                        module: "core".into(),
                        message: format!("Switched mode to {mode}"),
                    });
                    let _ = tx.send(UiEvent::Notify {
                        title: "ZeroClash".into(),
                        body: format!("Mode: {mode}"),
                    });
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "core".into(),
                        message: format!("Mode switch failed: {e}"),
                    });
                }
            }
        });
    }

    fn spawn_close_connection(&mut self, id: String) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        if self.selected_conn_id.as_deref() == Some(&id) {
            self.selected_conn_id = None;
        }
        crate::runtime::handle().spawn(async move {
            match client.close_connection(&id).await {
                Ok(()) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Info,
                        module: "conn".into(),
                        message: format!("Closed {id}"),
                    });
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "conn".into(),
                        message: format!("Failed to close {id}: {e}"),
                    });
                }
            }
        });
    }

    fn spawn_test_delay(&self, name: String) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let tx = self.event_tx.clone();
        crate::runtime::handle().spawn(async move {
            match client
                .proxy_delay(&name, 5000, "https://www.gstatic.com/generate_204")
                .await
            {
                Ok(delay) => {
                    let _ = tx.send(UiEvent::DelayMeasured {
                        name,
                        delay_ms: delay,
                    });
                }
                Err(e) => {
                    let _ = tx.send(UiEvent::Log {
                        level: LogLevel::Error,
                        module: "proxy".into(),
                        message: format!("Delay test {name} failed: {e}"),
                    });
                }
            }
        });
    }

    pub fn push_command(&mut self, cmd: UiCommand) {
        self.pending_commands.push(cmd);
    }

    fn process_commands(&mut self) {
        for cmd in std::mem::take(&mut self.pending_commands) {
            match cmd {
                UiCommand::Navigate(page) => self.current_page = page,
                UiCommand::RefreshProxies => self.spawn_refresh_proxies(),
                UiCommand::RefreshConnections => self.spawn_refresh_connections(),
                UiCommand::ToggleSystemProxy => {
                    self.enable_system_proxy = !self.enable_system_proxy;
                    self.config.verge.edit_draft(|v| {
                        v.enable_system_proxy = self.enable_system_proxy;
                    });
                    self.config.verge.apply();
                    self.save_config();
                    if self.enable_system_proxy {
                        if SystemProxy::enable(7899, 7898).is_ok() {
                            notify("ZeroClash", "System proxy enabled");
                        }
                    } else if SystemProxy::disable().is_ok() {
                        notify("ZeroClash", "System proxy disabled");
                    }
                }
                UiCommand::ToggleAutoStart => {
                    let verge = self.config.verge.latest_arc();
                    let enable = !verge.enable_auto_start;
                    self.config
                        .verge
                        .edit_draft(|v| v.enable_auto_start = enable);
                    self.config.verge.apply();
                    self.save_config();
                    log::info!("AutoStart toggled: {enable}");
                }
                UiCommand::ImportProfile(url) => self.handle_import_profile(url),
                UiCommand::ActivateProfile(uid) => self.handle_activate_profile(uid),
                UiCommand::DeleteProfile(uid) => self.handle_delete_profile(uid),
                UiCommand::CloseConnection(id) => self.spawn_close_connection(id),
                UiCommand::SelectProxy(group, proxy) => self.spawn_select_proxy(group, proxy),
                UiCommand::ToggleCore => self.handle_toggle_core(),
                UiCommand::SwitchMode(mode) => self.spawn_switch_mode(mode),
                UiCommand::ToggleTun => self.handle_toggle_tun(),
                UiCommand::TestDelay(name) => self.spawn_test_delay(name),
            }
        }
    }

    fn poll_events(&mut self) {
        let mut tray_events: Vec<crate::tray::TrayEvent> = Vec::new();
        if let Some(ref tray) = self.tray {
            while let Some(ev) = tray.poll() {
                tray_events.push(ev);
            }
        }
        for ev in tray_events {
            match ev {
                crate::tray::TrayEvent::Quit => {
                    log::info!("Tray: quit");
                    std::process::exit(0);
                }
                crate::tray::TrayEvent::SwitchMode(mode) => {
                    self.push_command(UiCommand::SwitchMode(mode));
                }
                crate::tray::TrayEvent::ToggleProxy => {
                    self.push_command(UiCommand::ToggleSystemProxy);
                }
                crate::tray::TrayEvent::ToggleTun => {
                    self.push_command(UiCommand::ToggleTun);
                }
                crate::tray::TrayEvent::ShowWindow => {
                    self.want_activate = true;
                }
            }
        }
        while let Some(action) = self.hotkey.poll() {
            match action {
                crate::hotkey::HotkeyAction::ToggleProxy => {
                    self.push_command(UiCommand::ToggleSystemProxy);
                }
                crate::hotkey::HotkeyAction::ShowWindow => {
                    self.want_activate = true;
                }
            }
        }
    }

    fn tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        if self.core_running && self.frame_count.is_multiple_of(30) {
            self.push_command(UiCommand::RefreshProxies);
            self.push_command(UiCommand::RefreshConnections);
        }
    }
}

impl Render for AppState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.tick();
        self.drain_events();
        self.process_commands();
        self.poll_events();

        if self.want_activate {
            window.activate_window();
            self.want_activate = false;
        }

        let theme = cx.global::<Theme>();
        let c = theme.colors;
        let cr = self.core_running;

        let status_color = if cr { c.success } else { c.text_muted };
        let status_key = if cr {
            "ui.sidebar.coreRunning"
        } else {
            "ui.sidebar.coreStopped"
        };
        let status_text = tr(status_key);
        let dot = if cr { "●" } else { "○" };

        let cp = self.current_page.clone();
        let sidebar_focus = self.sidebar_focus.clone();

        div()
            .size_full()
            .flex()
            .font_family(crate::fonts::sans_family())
            .bg(c.bg)
            .text_color(c.text_primary)
            .track_focus(&sidebar_focus)
            .key_context("AppRoot")
            .on_key_down(cx.listener(handle_root_key_down))
            .child(render_sidebar(
                cr,
                &cp,
                c,
                status_color,
                status_text,
                dot,
                cx,
            ))
            .child(render_content(cp, self, window, cx))
    }
}

fn render_content(
    page: Page,
    state: &mut AppState,
    window: &mut Window,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    div().flex_1().flex().child(match page {
        Page::Home => dashboard::dashboard_page(state, window, cx).into_any_element(),
        Page::Proxies => proxies::proxies_page(state, window, cx).into_any_element(),
        Page::Profiles => profiles::profiles_page(state, window, cx).into_any_element(),
        Page::Connections => connections::connections_page(state, window, cx).into_any_element(),
        Page::Logs => logs::logs_page(state, window, cx).into_any_element(),
        Page::Settings => settings::settings_page(state, window, cx).into_any_element(),
    })
}

fn render_sidebar(
    _core_running: bool,
    current_page: &Page,
    c: Colors,
    status_color: gpui::Hsla,
    status_text: gpui::SharedString,
    dot: &str,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let nav_pages: Vec<(gpui::SharedString, Page)> = vec![
        (tr("ui.nav.dashboard"), Page::Home),
        (tr("ui.nav.proxies"), Page::Proxies),
        (tr("ui.nav.profiles"), Page::Profiles),
        (tr("ui.nav.connections"), Page::Connections),
        (tr("ui.nav.logs"), Page::Logs),
        (tr("ui.nav.settings"), Page::Settings),
    ];

    let cp = current_page.clone();

    div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(c.sidebar_bg)
        .px(px(SPACE_LG))
        .py(px(SPACE_LG))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_color(white()).child(tr("ui.sidebar.appName")))
                .child(
                    div()
                        .mt(px(SPACE_XS))
                        .text_color(status_color)
                        .child(gpui::SharedString::from(format!("{dot} {status_text}"))),
                ),
        )
        .child(
            div()
                .mt(px(SPACE_LG))
                .text_color(c.sidebar_text_muted)
                .child(tr("ui.sidebar.section")),
        )
        .child(
            div()
                .mt(px(SPACE_SM))
                .flex()
                .flex_col()
                .children(nav_pages.into_iter().map(|(label, page)| {
                    let active = cp == page;
                    let text = if active { c.accent } else { c.sidebar_text };
                    let bg = if active {
                        c.sidebar_active_bg
                    } else {
                        gpui::transparent_black()
                    };
                    let p = page;
                    div()
                        .flex()
                        .items_center()
                        .h(px(32.0))
                        .px(px(SPACE_MD))
                        .rounded(px(design::RADIUS_SM))
                        .bg(bg)
                        .text_color(text)
                        .cursor(CursorStyle::PointingHand)
                        .child(label)
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _e, _w, cx| {
                                this.current_page = p.clone();
                                cx.notify();
                            }),
                        )
                })),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_end()
                .child(div().text_color(c.sidebar_text_muted).child(VERSION_LABEL)),
        )
}

const VERSION_LABEL: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Root-level keyboard navigation. Handles vertical arrow keys to move
/// between sidebar pages and Cmd+number to jump directly to a page.
fn handle_root_key_down(
    state: &mut AppState,
    event: &KeyDownEvent,
    _window: &mut Window,
    cx: &mut Context<AppState>,
) {
    let ks = &event.keystroke;

    // Skip if a child element (logs search, proxies filter, import
    // dialog) currently owns focus — those views handle their own keys.
    if !ks.modifiers.platform && !ks.modifiers.control && !ks.modifiers.alt {
        match ks.key.as_str() {
            "down" => {
                state.cycle_page(1);
                cx.notify();
                return;
            }
            "up" => {
                state.cycle_page(-1);
                cx.notify();
                return;
            }
            _ => {}
        }
    }

    if ks.modifiers.platform
        && let Ok(idx) = ks.key.parse::<u32>()
        && (1..=6).contains(&idx)
    {
        let pages = [
            Page::Home,
            Page::Proxies,
            Page::Profiles,
            Page::Connections,
            Page::Logs,
            Page::Settings,
        ];
        state.current_page = pages[(idx - 1) as usize].clone();
        cx.notify();
    }
}
