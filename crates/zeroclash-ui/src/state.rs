use gpui::{Context, CursorStyle, MouseButton, Render, Window, div, prelude::*, px, white};
use zeroclash_core::MihomoClient;
use zeroclash_core::mihomo::ProxyGroup;
use zeroclash_core::profile::ProfilePreview;
use zeroclash_core::{ConnEntry, SystemProxy, notify};

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
    pending_commands: Vec<UiCommand>,
    tray: Option<TrayManager>,
    hotkey: HotkeyManager,
    client: Option<MihomoClient>,
    frame_count: u64,
}

impl AppState {
    pub fn new(tray: Option<TrayManager>, hotkey: HotkeyManager) -> Self {
        let mut lv = LogViewer::default();
        lv.store.push(
            crate::components::log_viewer::LogLevel::Info,
            "zeroclash",
            "Application started",
        );
        Self {
            current_page: Page::Home,
            core_running: false,
            traffic: TrafficHistory::default(),
            proxy_groups: Vec::new(),
            connections: Vec::new(),
            enable_system_proxy: false,
            profile_previews: Vec::new(),
            import_dialog_visible: false,
            import_url: String::new(),
            selected_conn_id: None,
            log_viewer: lv,
            pending_commands: Vec::new(),
            tray,
            hotkey,
            client: None,
            frame_count: 0,
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
                    if self.enable_system_proxy {
                        if SystemProxy::enable(7899, 7898).is_ok() {
                            notify("ZeroClash", "System proxy enabled");
                        }
                    } else if SystemProxy::disable().is_ok() {
                        notify("ZeroClash", "System proxy disabled");
                    }
                }
                UiCommand::ToggleAutoStart => {
                    log::info!("ToggleAutoStart");
                }
                UiCommand::ImportProfile(url) => {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Info,
                        "profile",
                        &format!("Importing {url}"),
                    );
                }
                UiCommand::ActivateProfile(uid) => {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Info,
                        "profile",
                        &format!("Activated {uid}"),
                    );
                }
                UiCommand::DeleteProfile(uid) => {
                    self.log_viewer.store.push(
                        crate::components::log_viewer::LogLevel::Info,
                        "profile",
                        &format!("Deleted {uid}"),
                    );
                }
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
                UiCommand::ToggleCore => {
                    self.core_running = !self.core_running;
                    if self.core_running {
                        self.client = Some(MihomoClient::default_addr());
                        self.log_viewer.store.push(
                            crate::components::log_viewer::LogLevel::Info,
                            "core",
                            "Core started",
                        );
                        notify("ZeroClash", "Core started");
                    } else {
                        self.client = None;
                        self.proxy_groups.clear();
                        self.connections.clear();
                        self.log_viewer.store.push(
                            crate::components::log_viewer::LogLevel::Info,
                            "core",
                            "Core stopped",
                        );
                        notify("ZeroClash", "Core stopped");
                    }
                }
            }
        }
    }

    fn poll_events(&mut self) {
        if let Some(ref tray) = self.tray
            && tray.poll().is_some()
        {
            log::info!("Tray event received");
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
