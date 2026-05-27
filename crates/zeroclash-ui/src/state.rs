use gpui::{Context, CursorStyle, MouseButton, Render, Window, div, prelude::*, px, white};
use std::path::PathBuf;
use zeroclash_core::CoreManager;
use zeroclash_core::MihomoClient;
use zeroclash_core::config::VergeConfig;
use zeroclash_core::mihomo::ProxyGroup;
use zeroclash_core::profile::{ProfilePreview, ProfileStore};
use zeroclash_core::{Config, ConnEntry, SystemProxy, notify};

use crate::components::log_viewer::LogViewer;
use crate::components::traffic_graph::TrafficHistory;
use crate::design::{self, Colors, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS};
use crate::hotkey::HotkeyManager;
use crate::theme::Theme;
use crate::tray::TrayManager;
use crate::util;
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

pub struct AppState {
    pub current_page: Page,
    pub core_running: bool,
    pub traffic: TrafficHistory,
    pub proxy_groups: Vec<ProxyGroup>,
    pub connections: Vec<ConnEntry>,
    pub enable_system_proxy: bool,
    pub profile_previews: Vec<ProfilePreview>,
    pub import_dialog_visible: bool,
    pub import_url: String,
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
}

impl AppState {
    pub fn new(data_dir: PathBuf, tray: Option<TrayManager>, hotkey: HotkeyManager) -> Self {
        let mut lv = LogViewer::default();
        lv.store.push(
            crate::components::log_viewer::LogLevel::Info,
            "zeroclash",
            "Application started",
        );

        let config = Self::load_config(&data_dir);
        let enable_system_proxy = config.verge.latest_arc().enable_system_proxy;
        let profile_store = pollster::block_on(ProfileStore::load(data_dir.clone())).ok();
        let profile_previews = profile_store
            .as_ref()
            .map(|ps| ps.preview())
            .unwrap_or_default();

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
        }
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
        match pollster::block_on(ps.fetch_remote(&url, None, None)) {
            Ok(item) => {
                if let Err(e) = ps.add_item(item) {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Error,
                        "profile",
                        &format!("Failed to add: {e}"),
                    );
                }
                if let Err(e) = pollster::block_on(ps.save()) {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Error,
                        "profile",
                        &format!("Failed to save: {e}"),
                    );
                }
                self.refresh_profiles();
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Info,
                    "profile",
                    &format!("Imported {url}"),
                );
                notify("ZeroClash", "Profile imported");
            }
            Err(e) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
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
                crate::components::log_viewer::LogLevel::Error,
                "profile",
                &format!("Failed to activate: {e}"),
            );
        } else {
            let _ = pollster::block_on(ps.save());
            self.refresh_profiles();
            self.log_viewer.store.push(
                crate::components::log_viewer::LogLevel::Info,
                "profile",
                &format!("Activated {uid}"),
            );
            notify("ZeroClash", "Profile activated");
        }
    }

    fn handle_delete_profile(&mut self, uid: String) {
        let Some(ref mut ps) = self.profile_store else {
            return;
        };
        match ps.delete_item(&uid) {
            Ok(_) => {
                let _ = pollster::block_on(ps.save());
                self.refresh_profiles();
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Info,
                    "profile",
                    &format!("Deleted {uid}"),
                );
                notify("ZeroClash", "Profile deleted");
            }
            Err(e) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
                    "profile",
                    &format!("Failed to delete: {e}"),
                );
            }
        }
    }

    fn handle_select_proxy(&mut self, group: String, proxy: String) {
        let Some(ref c) = self.client else {
            return;
        };
        match pollster::block_on(c.select_proxy(&group, &proxy)) {
            Ok(()) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Info,
                    "proxy",
                    &format!("Switched {group} -> {proxy}"),
                );
                self.push_command(UiCommand::RefreshProxies);
            }
            Err(e) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
                    "proxy",
                    &format!("Switch failed: {e}"),
                );
            }
        }
    }

    fn handle_toggle_core(&mut self) {
        if self.core_running {
            if let Some(ref cm) = self.core_manager
                && let Err(e) = pollster::block_on(cm.stop())
            {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
                    "core",
                    &format!("Failed to stop: {e}"),
                );
            }
            self.core_running = false;
            self.client = None;
            self.core_manager = None;
            self.proxy_groups.clear();
            self.connections.clear();
            self.log_viewer.store.push(
                crate::components::log_viewer::LogLevel::Info,
                "core",
                "Core stopped",
            );
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
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Info,
                        "core",
                        &format!("Core started from {core_path}"),
                    );
                    notify("ZeroClash", "Core started");
                }
                Err(e) => {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Error,
                        "core",
                        &format!("Failed to start: {e}"),
                    );
                }
            }
        }
    }

    fn handle_switch_mode(&mut self, mode: String) {
        let Some(ref c) = self.client else {
            return;
        };
        let mode_str = mode.clone();
        match pollster::block_on(c.switch_mode(&mode)) {
            Ok(()) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Info,
                    "core",
                    &format!("Switched mode to {mode_str}"),
                );
                notify("ZeroClash", &format!("Mode: {mode_str}"));
            }
            Err(e) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
                    "core",
                    &format!("Mode switch failed: {e}"),
                );
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
            crate::components::log_viewer::LogLevel::Info,
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
    }

    fn handle_test_delay(&mut self, name: String) {
        let Some(ref c) = self.client else {
            return;
        };
        match pollster::block_on(c.proxy_delay(&name, 5000, "https://www.gstatic.com/generate_204"))
        {
            Ok(delay) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Info,
                    "proxy",
                    &format!("{name}: {delay}ms"),
                );
                self.push_command(UiCommand::RefreshProxies);
            }
            Err(e) => {
                self.log_viewer.store.push(
                    crate::components::log_viewer::LogLevel::Error,
                    "proxy",
                    &format!("Delay test {name} failed: {e}"),
                );
            }
        }
    }

    pub fn push_command(&mut self, cmd: UiCommand) {
        self.pending_commands.push(cmd);
    }

    fn process_commands(&mut self) {
        for cmd in std::mem::take(&mut self.pending_commands) {
            match cmd {
                UiCommand::Navigate(page) => self.current_page = page,
                UiCommand::RefreshProxies => {
                    if let Some(ref c) = self.client
                        && let Ok(v) = pollster::block_on(c.proxies())
                    {
                        self.proxy_groups = util::parse_proxy_groups(&v);
                    }
                }
                UiCommand::RefreshConnections => {
                    if let Some(ref c) = self.client
                        && let Ok(v) = pollster::block_on(c.connections())
                    {
                        self.connections = util::parse_connections(&v);
                    }
                }
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
                UiCommand::CloseConnection(id) => {
                    if let Some(ref c) = self.client {
                        let _ = pollster::block_on(c.close_connection(&id));
                    }
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Info,
                        "conn",
                        &format!("Closed {id}"),
                    );
                    if self.selected_conn_id.as_deref() == Some(&id) {
                        self.selected_conn_id = None;
                    }
                }
                UiCommand::SelectProxy(group, proxy) => self.handle_select_proxy(group, proxy),
                UiCommand::ToggleCore => self.handle_toggle_core(),
                UiCommand::SwitchMode(mode) => self.handle_switch_mode(mode),
                UiCommand::ToggleTun => self.handle_toggle_tun(),
                UiCommand::TestDelay(name) => self.handle_test_delay(name),
            }
        }
    }

    fn poll_events(&mut self) {
        if let Some(ref tray) = self.tray {
            match tray.poll() {
                Some(crate::tray::TrayEvent::Quit) => {
                    log::info!("Tray: quit");
                    std::process::exit(0);
                }
                Some(crate::tray::TrayEvent::SwitchMode(mode)) => {
                    self.push_command(UiCommand::SwitchMode(mode));
                }
                Some(crate::tray::TrayEvent::ToggleProxy) => {
                    self.push_command(UiCommand::ToggleSystemProxy);
                }
                Some(crate::tray::TrayEvent::ToggleTun) => {
                    self.push_command(UiCommand::ToggleTun);
                }
                Some(crate::tray::TrayEvent::ShowWindow) => {
                    log::info!("Tray: show window");
                }
                None => {}
            }
        }
        if self.hotkey.poll().is_some() {
            self.push_command(UiCommand::ToggleSystemProxy);
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
        self.process_commands();
        self.poll_events();

        let theme = cx.global::<Theme>();
        let c = theme.colors;
        let cr = self.core_running;

        let status_color = if cr { c.success } else { c.text_muted };
        let status_text = if cr { "Core Running" } else { "Core Stopped" };
        let dot = if cr { "●" } else { "○" };

        let cp = self.current_page.clone();

        div()
            .size_full()
            .flex()
            .bg(c.bg)
            .text_color(c.text_primary)
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
    status_text: &str,
    dot: &str,
    cx: &mut Context<AppState>,
) -> impl IntoElement {
    let nav_pages: Vec<(&str, Page)> = vec![
        ("Dashboard", Page::Home),
        ("Proxies", Page::Proxies),
        ("Profiles", Page::Profiles),
        ("Connections", Page::Connections),
        ("Logs", Page::Logs),
        ("Settings", Page::Settings),
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
                .child(div().text_color(white()).child("ZeroClash"))
                .child(
                    div()
                        .mt(px(SPACE_XS))
                        .text_color(status_color)
                        .child(format!("{dot} {status_text}")),
                ),
        )
        .child(
            div()
                .mt(px(SPACE_LG))
                .text_color(c.sidebar_text_muted)
                .child("NAVIGATION"),
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
            div().flex_1().flex().flex_col().justify_end().child(
                div()
                    .text_color(c.sidebar_text_muted)
                    .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
            ),
        )
}
