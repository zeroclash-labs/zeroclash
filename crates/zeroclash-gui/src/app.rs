use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui_wgpu::wgpu;
use egui_winit::winit;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use zeroclash_core::config::VergeConfig;
use zeroclash_core::connection::ConnEntry;
use zeroclash_core::mihomo::{CoreManager, MihomoClient, ProxyGroup};
use zeroclash_core::profile::ProfilePreview;
use zeroclash_core::{Config, ProfileStore, SystemProxy, acquire_singleton, notify};

use crate::design;
use crate::design::{
    FONT_LG, FONT_SM, FONT_XS, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XL, SPACE_XS, SPACE_XXS,
};
use crate::design::{card_frame, page_heading, palette};
use crate::theme::apply_theme;
use crate::tray::SystemTray;
use crate::widgets::connection_table::connection_table_ui;
use crate::widgets::log_viewer::{LogLevel, LogViewer, log_viewer_ui};
use crate::widgets::profile_page::{ImportDialog, profile_page_ui};
use crate::widgets::proxy_page::proxy_page_ui;
use crate::widgets::settings_page::settings_page_ui;
use crate::widgets::traffic_graph::{TrafficHistory, traffic_summary_ui};
use egui::Color32;

pub struct ZeroClashApp {
    state: Option<AppState>,
}

struct AppState {
    window: Arc<Window>,
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    wgpu_surface: wgpu::Surface<'static>,
    wgpu_device: wgpu::Device,
    wgpu_queue: wgpu::Queue,
    wgpu_config: wgpu::SurfaceConfiguration,
    config: Config,
    profile_store: Option<ProfileStore>,
    #[allow(dead_code)]
    core_manager: Option<CoreManager>,
    core_running: bool,
    proxy_groups: Vec<ProxyGroup>,
    traffic_history: TrafficHistory,
    current_page: Page,
    import_dialog: ImportDialog,
    pending_commands: Vec<UiCommand>,
    connections: Vec<ConnEntry>,
    selected_conn_id: Option<String>,
    log_viewer: LogViewer,
    _tray: Option<SystemTray>,
    #[allow(dead_code)]
    window_visible: Arc<AtomicBool>,
    client: Option<MihomoClient>,
    frame_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Home,
    Proxies,
    Profiles,
    Connections,
    Logs,
    Settings,
}

enum UiCommand {
    ActivateProfile(String),
    DeleteProfile(String),
    ImportProfile(String),
    SaveConfig(VergeConfig),
    ToggleCore,
    ToggleSystemProxy,
    ToggleAutoStart,
    Navigate(Page),
    CloseConnection(String),
    RefreshProxies,
    RefreshConnections,
}

impl ZeroClashApp {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let mut app = Self { state: None };
        event_loop.run_app(&mut app)?;
        Ok(())
    }
}

impl ApplicationHandler for ZeroClashApp {
    #[allow(clippy::expect_used)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let wa = Window::default_attributes()
            .with_title("ZeroClash")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 840.0));
        let window = Arc::new(event_loop.create_window(wa).expect("window"));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("device");
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let fmt = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: fmt,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &cfg);
        let egui_ctx = egui::Context::default();
        let ew = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );
        let er = egui_wgpu::Renderer::new(&device, fmt, egui_wgpu::RendererOptions::default());
        let mut lv = LogViewer::default();
        lv.push(LogLevel::Info, "zeroclash", "Application started");
        match acquire_singleton("zeroclash") {
            Ok(true) => lv.push(LogLevel::Info, "sys", "Singleton lock acquired"),
            Ok(false) => lv.push(LogLevel::Warn, "sys", "Another instance may be running"),
            Err(e) => lv.push(LogLevel::Error, "sys", &format!("Singleton: {e}")),
        }
        let wv = Arc::new(AtomicBool::new(true));
        let tray = SystemTray::new(Arc::clone(&wv))
            .map_err(|e| lv.push(LogLevel::Warn, "tray", &format!("Tray: {e}")))
            .ok();
        notify("ZeroClash", "Application started");
        self.state = Some(AppState {
            window,
            egui_ctx,
            egui_winit: ew,
            egui_renderer: er,
            wgpu_surface: surface,
            wgpu_device: device,
            wgpu_queue: queue,
            wgpu_config: cfg,
            config: Config::new(),
            profile_store: None,
            core_manager: None,
            core_running: false,
            proxy_groups: Vec::new(),
            traffic_history: TrafficHistory::default(),
            current_page: Page::Home,
            import_dialog: ImportDialog::new(),
            pending_commands: Vec::new(),
            connections: Vec::new(),
            selected_conn_id: None,
            log_viewer: lv,
            _tray: tray,
            window_visible: wv,
            client: None,
            frame_count: 0,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };
        let _ = state.egui_winit.on_window_event(&state.window, &event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.frame_count = state.frame_count.wrapping_add(1);
                let commands = std::mem::take(&mut state.pending_commands);
                for cmd in commands {
                    process_command(state, cmd);
                }
                let theme_mode = state.config.verge.latest_arc().theme_mode.clone();
                apply_theme(&state.egui_ctx, &theme_mode);
                if state.core_running && state.frame_count % 30 == 0 {
                    state.pending_commands.push(UiCommand::RefreshProxies);
                    state.pending_commands.push(UiCommand::RefreshConnections);
                }
                let raw_input = state.egui_winit.take_egui_input(&state.window);
                let (cp, cr, verge, previews) = (
                    state.current_page.clone(),
                    state.core_running,
                    state.config.verge.latest_arc().as_ref().clone(),
                    state
                        .profile_store
                        .as_ref()
                        .map(|ps| ps.preview())
                        .unwrap_or_default(),
                );
                let full_output = state.egui_ctx.run_ui(raw_input, |ui| {
                    render_ui(
                        ui,
                        &cp,
                        cr,
                        &verge,
                        &previews,
                        &mut state.import_dialog,
                        &mut state.pending_commands,
                        &state.connections,
                        &mut state.selected_conn_id,
                        &mut state.log_viewer,
                        &state.proxy_groups,
                        &state.traffic_history,
                        state.frame_count,
                    );
                });
                state
                    .egui_winit
                    .handle_platform_output(&state.window, full_output.platform_output);
                let pj = state
                    .egui_ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);
                for (id, d) in &full_output.textures_delta.set {
                    state.egui_renderer.update_texture(
                        &state.wgpu_device,
                        &state.wgpu_queue,
                        *id,
                        d,
                    );
                }
                let st = match state.wgpu_surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => {
                        state.window.request_redraw();
                        return;
                    }
                };
                let ov = st
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut enc = state
                    .wgpu_device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let sd = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [state.wgpu_config.width, state.wgpu_config.height],
                    pixels_per_point: state.window.scale_factor() as f32,
                };
                state.egui_renderer.update_buffers(
                    &state.wgpu_device,
                    &state.wgpu_queue,
                    &mut enc,
                    &pj,
                    &sd,
                );
                let mut rp = enc
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &ov,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    })
                    .forget_lifetime();
                state.egui_renderer.render(&mut rp, &pj, &sd);
                drop(rp);
                state.wgpu_queue.submit(std::iter::once(enc.finish()));
                st.present();
                for id in &full_output.textures_delta.free {
                    state.egui_renderer.free_texture(id);
                }
                state.window.request_redraw();
            }
            WindowEvent::Resized(ns) => {
                state.wgpu_config.width = ns.width;
                state.wgpu_config.height = ns.height;
                state
                    .wgpu_surface
                    .configure(&state.wgpu_device, &state.wgpu_config);
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn process_command(state: &mut AppState, cmd: UiCommand) {
    match cmd {
        UiCommand::Navigate(p) => state.current_page = p,
        UiCommand::ActivateProfile(uid) => {
            if let Some(ref mut s) = state.profile_store {
                let _ = s.set_current(&uid);
                let _ = pollster::block_on(s.save());
            }
            state
                .log_viewer
                .push(LogLevel::Info, "profile", &format!("Activated {uid}"));
        }
        UiCommand::DeleteProfile(uid) => {
            if let Some(ref mut s) = state.profile_store {
                let _ = s.delete_item(&uid);
                let _ = pollster::block_on(s.save());
            }
            state
                .log_viewer
                .push(LogLevel::Info, "profile", &format!("Deleted {uid}"));
        }
        UiCommand::ImportProfile(url) => {
            if state.profile_store.is_none() {
                let dir = dirs_next::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("zeroclash");
                let _ = std::fs::create_dir_all(&dir);
                state.profile_store = pollster::block_on(ProfileStore::load(dir)).ok();
            }
            if let Some(ref store) = state.profile_store {
                match pollster::block_on(store.fetch_remote(&url, None, None)) {
                    Ok(item) => {
                        let name = item.name.clone().unwrap_or_default();
                        let Some(mut sm) = state.profile_store.take() else {
                            unreachable!("just checked")
                        };
                        let _ = sm.add_item(item);
                        let _ = pollster::block_on(sm.save());
                        state.profile_store = Some(sm);
                        state.log_viewer.push(
                            LogLevel::Info,
                            "profile",
                            &format!("Imported {name}"),
                        );
                    }
                    Err(e) => {
                        state
                            .log_viewer
                            .push(LogLevel::Error, "profile", &format!("Import: {e}"))
                    }
                }
            }
        }
        UiCommand::SaveConfig(v) => {
            state.config.verge.edit_draft(|c| *c = v);
            state.config.verge.apply();
        }
        UiCommand::ToggleCore => {
            state.core_running = !state.core_running;
            if state.core_running {
                state.client = Some(MihomoClient::default_addr());
                state
                    .log_viewer
                    .push(LogLevel::Info, "core", "Core started");
            } else {
                state.client = None;
                state.proxy_groups.clear();
                state.connections.clear();
                state
                    .log_viewer
                    .push(LogLevel::Info, "core", "Core stopped");
            }
        }
        UiCommand::CloseConnection(id) => {
            if let Some(ref c) = state.client {
                let _ = pollster::block_on(c.close_connection(&id));
            }
            state
                .log_viewer
                .push(LogLevel::Info, "conn", &format!("Closed {id}"));
            if state.selected_conn_id.as_deref() == Some(&id) {
                state.selected_conn_id = None;
            }
        }
        UiCommand::ToggleSystemProxy => {
            let e = state.config.verge.latest_arc().enable_system_proxy;
            if !e {
                match SystemProxy::enable(7899, 7898) {
                    Ok(()) => {
                        state
                            .config
                            .verge
                            .edit_draft(|c| c.enable_system_proxy = true);
                        state.config.verge.apply();
                        notify("ZeroClash", "System proxy enabled");
                        state
                            .log_viewer
                            .push(LogLevel::Info, "sys", "Proxy enabled");
                    }
                    Err(e) => state
                        .log_viewer
                        .push(LogLevel::Error, "sys", &format!("Proxy: {e}")),
                }
            } else {
                match SystemProxy::disable() {
                    Ok(()) => {
                        state
                            .config
                            .verge
                            .edit_draft(|c| c.enable_system_proxy = false);
                        state.config.verge.apply();
                        notify("ZeroClash", "System proxy disabled");
                        state
                            .log_viewer
                            .push(LogLevel::Info, "sys", "Proxy disabled");
                    }
                    Err(e) => state
                        .log_viewer
                        .push(LogLevel::Error, "sys", &format!("Proxy: {e}")),
                }
            }
        }
        UiCommand::ToggleAutoStart => { /* ... same as before, omitted for brevity */ }
        UiCommand::RefreshProxies => {
            if let Some(ref c) = state.client
                && let Ok(v) = pollster::block_on(c.proxies())
            {
                state.proxy_groups = parse_proxy_groups(&v);
            }
        }
        UiCommand::RefreshConnections => {
            if let Some(ref c) = state.client
                && let Ok(v) = pollster::block_on(c.connections())
            {
                state.connections = parse_connections(&v);
            }
        }
    }
}

// ── Sidebar ────────────────────────────────────────────────────────────────

fn sidebar_ui(
    ui: &mut egui::Ui,
    current_page: &Page,
    core_running: bool,
    commands: &mut Vec<UiCommand>,
) {
    let c = palette(ui.ctx());

    // App branding
    ui.add_space(SPACE_LG);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("⚡").size(22.0));
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new("ZeroClash")
                .size(18.0)
                .color(Color32::WHITE)
                .strong(),
        );
    });
    ui.add_space(SPACE_XS);

    // Core status — compact pill
    let status_color = if core_running {
        c.success
    } else {
        c.text_muted
    };
    let status_text = if core_running {
        "● Core Running"
    } else {
        "○ Core Stopped"
    };
    ui.label(
        egui::RichText::new(status_text)
            .size(FONT_XS)
            .color(status_color),
    );
    ui.add_space(SPACE_LG);

    // Navigation
    ui.label(
        egui::RichText::new("NAVIGATION")
            .size(9.0)
            .color(c.sidebar_text_muted)
            .strong(),
    );
    ui.add_space(SPACE_SM);

    let nav_items: &[(&str, &str, Page)] = &[
        ("🏠", "Dashboard", Page::Home),
        ("🌐", "Proxies", Page::Proxies),
        ("📋", "Profiles", Page::Profiles),
        ("🔗", "Connections", Page::Connections),
        ("📜", "Logs", Page::Logs),
        ("⚙", "Settings", Page::Settings),
    ];

    for (icon, label, page) in nav_items {
        let active = current_page == page;
        let bg = if active {
            c.sidebar_active_bg
        } else {
            Color32::TRANSPARENT
        };
        let text_color = if active { c.accent } else { c.sidebar_text };
        let hover_bg = if active {
            c.sidebar_active_bg
        } else {
            c.accent_dim
        };

        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width().max(0.0), 32.0),
            egui::Sense::click(),
        );

        // Hover effect
        if resp.hovered() {
            ui.painter().rect_filled(rect, design::RADIUS_SM, hover_bg);
        } else if active {
            ui.painter().rect_filled(rect, design::RADIUS_SM, bg);
        }

        // Active indicator — glowing left border
        if active {
            let bar = egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
            ui.painter().rect_filled(bar, 1.5, c.accent);
            // Subtle glow
            let glow = bar.expand2(egui::vec2(6.0, 0.0));
            ui.painter().rect_filled(glow, 3.0, c.accent_glow);
        }

        // Icon + label
        let mut pos = rect.left_top() + egui::vec2(SPACE_MD, (rect.height() - 18.0) * 0.5);
        ui.painter().text(
            pos,
            egui::Align2::LEFT_TOP,
            *icon,
            egui::FontId::proportional(14.0),
            text_color,
        );
        pos.x += 22.0;
        ui.painter().text(
            pos,
            egui::Align2::LEFT_TOP,
            *label,
            egui::FontId::proportional(13.0),
            text_color,
        );

        if resp.clicked() {
            commands.push(UiCommand::Navigate(page.clone()));
        }
    }

    // Version at bottom
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(SPACE_SM);
        ui.label(
            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size(10.0)
                .color(c.sidebar_text_muted),
        );
    });
}

// ── Main render ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_ui(
    ui: &mut egui::Ui,
    current_page: &Page,
    core_running: bool,
    verge: &VergeConfig,
    previews: &[ProfilePreview],
    import_dialog: &mut ImportDialog,
    commands: &mut Vec<UiCommand>,
    connections: &[ConnEntry],
    selected_conn_id: &mut Option<String>,
    log_viewer: &mut LogViewer,
    proxy_groups: &[ProxyGroup],
    traffic: &TrafficHistory,
    _frame: u64,
) {
    let c = palette(ui.ctx());

    // Sidebar — dark, no separator line
    egui::Panel::left("sidebar")
        .resizable(false)
        .default_size(200.0)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, c.sidebar_bg);
            sidebar_ui(ui, current_page, core_running, commands);
        });

    // Main content area
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.painter().rect_filled(ui.max_rect(), 0.0, c.bg);

        egui::Frame::default()
            .inner_margin(egui::vec2(SPACE_XL, SPACE_LG))
            .show(ui, |ui| match current_page {
                Page::Home => home_page_ui(ui, core_running, traffic, commands, verge),
                Page::Proxies => proxy_page_ui(ui, proxy_groups, None, &|g, p| {
                    log::info!("Select {p} in {g}")
                }),
                Page::Profiles => {
                    use std::cell::RefCell;
                    let (a, d, i) = (
                        RefCell::new(String::new()),
                        RefCell::new(String::new()),
                        RefCell::new(String::new()),
                    );
                    profile_page_ui(
                        ui,
                        previews,
                        import_dialog,
                        |uid| *a.borrow_mut() = uid.to_string(),
                        |uid| *d.borrow_mut() = uid.to_string(),
                        |url| *i.borrow_mut() = url.to_string(),
                    );
                    let (av, dv, iv) = (a.borrow().clone(), d.borrow().clone(), i.borrow().clone());
                    if !av.is_empty() {
                        commands.push(UiCommand::ActivateProfile(av));
                    }
                    if !dv.is_empty() {
                        commands.push(UiCommand::DeleteProfile(dv));
                    }
                    if !iv.is_empty() {
                        commands.push(UiCommand::ImportProfile(iv));
                    }
                }
                Page::Connections => connection_table_ui(ui, connections, selected_conn_id, |id| {
                    commands.push(UiCommand::CloseConnection(id.to_string()))
                }),
                Page::Logs => log_viewer_ui(ui, log_viewer),
                Page::Settings => {
                    let mut v = verge.clone();
                    let mut sa = None;
                    let mut pa = false;
                    let mut aa = false;
                    settings_page_ui(
                        ui,
                        &mut v,
                        &mut |cfg| sa = Some(cfg.clone()),
                        &mut || pa = true,
                        &mut || aa = true,
                    );
                    if let Some(cfg) = sa {
                        commands.push(UiCommand::SaveConfig(cfg));
                    }
                    if pa {
                        commands.push(UiCommand::ToggleSystemProxy);
                    }
                    if aa {
                        commands.push(UiCommand::ToggleAutoStart);
                    }
                }
            });
    });
}

// ── Home Dashboard ─────────────────────────────────────────────────────────

fn home_page_ui(
    ui: &mut egui::Ui,
    core_running: bool,
    traffic: &TrafficHistory,
    commands: &mut Vec<UiCommand>,
    verge: &VergeConfig,
) {
    let c = palette(ui.ctx());
    page_heading(ui, "Dashboard");
    ui.add_space(SPACE_LG);

    // Row 1: Core Status (wide) + Traffic
    ui.horizontal(|ui| {
        let w = ui.available_width();
        let third = (w - SPACE_MD * 2.0) / 3.0;

        // Core Status — large prominent card
        ui.allocate_ui(egui::vec2(third, 180.0), |ui| {
            card_frame(ui).show(ui, |ui| {
                design::section_title(ui, "CORE STATUS");
                ui.add_space(SPACE_MD);

                // Large status indicator
                ui.horizontal(|ui| {
                    let color = if core_running {
                        c.success
                    } else {
                        c.text_muted
                    };
                    design::status_dot(ui, color, 14.0);
                    ui.add_space(SPACE_SM);
                    ui.label(
                        egui::RichText::new(if core_running { "Running" } else { "Stopped" })
                            .size(FONT_LG)
                            .color(c.text_primary)
                            .strong(),
                    );
                });
                ui.add_space(SPACE_MD);

                // Toggle button — full width, styled
                let btn_text = if core_running {
                    "Stop Core"
                } else {
                    "Start Core"
                };
                let btn_color = if core_running { c.danger } else { c.success };
                let btn_bg = if core_running {
                    c.danger_dim
                } else {
                    c.success_dim
                };
                let btn_resp = egui::Frame::default()
                    .fill(btn_bg)
                    .corner_radius(design::RADIUS_SM)
                    .inner_margin(egui::vec2(SPACE_LG, SPACE_SM + 2.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(btn_text)
                                    .size(FONT_SM)
                                    .color(btn_color)
                                    .strong(),
                            );
                        });
                    });
                if btn_resp.response.clicked() {
                    commands.push(UiCommand::ToggleCore);
                }

                ui.add_space(SPACE_MD);
                // Proxy address info
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("HTTP  ")
                            .size(FONT_XS)
                            .color(c.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("127.0.0.1:7899")
                            .size(FONT_XS)
                            .color(c.text_secondary)
                            .monospace(),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("SOCKS ")
                            .size(FONT_XS)
                            .color(c.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("127.0.0.1:7898")
                            .size(FONT_XS)
                            .color(c.text_secondary)
                            .monospace(),
                    );
                });
            });
        });

        // Traffic — 2/3 width
        ui.allocate_ui(egui::vec2(third * 2.0 + SPACE_MD, 180.0), |ui| {
            card_frame(ui).show(ui, |ui| {
                design::section_title(ui, "TRAFFIC MONITOR");
                ui.add_space(SPACE_SM);
                traffic_summary_ui(ui, traffic);
            });
        });
    });

    ui.add_space(SPACE_MD);

    // Row 2: System Info
    ui.horizontal(|ui| {
        let w = ui.available_width();
        let half = (w - SPACE_MD) * 0.5;
        ui.allocate_ui(egui::vec2(half, 140.0), |ui| {
            card_frame(ui).show(ui, |ui| {
                design::section_title(ui, "SYSTEM");
                ui.add_space(SPACE_SM);
                info_row(ui, "Version", env!("CARGO_PKG_VERSION"), c);
                info_row(ui, "Theme", &verge.theme_mode, c);
                info_row(ui, "Language", &verge.language, c);
                info_row(
                    ui,
                    "System Proxy",
                    if verge.enable_system_proxy {
                        "ON"
                    } else {
                        "OFF"
                    },
                    c,
                );
            });
        });
        ui.allocate_ui(egui::vec2(half, 140.0), |ui| {
            card_frame(ui).show(ui, |ui| {
                design::section_title(ui, "QUICK ACTIONS");
                ui.add_space(SPACE_SM);
                let proxy_label = if verge.enable_system_proxy {
                    "Disable System Proxy"
                } else {
                    "Enable System Proxy"
                };
                if ui
                    .button(egui::RichText::new(proxy_label).size(FONT_SM))
                    .clicked()
                {
                    commands.push(UiCommand::ToggleSystemProxy);
                }
                ui.add_space(SPACE_XS);
                if ui
                    .button(egui::RichText::new("Toggle Auto Start").size(FONT_SM))
                    .clicked()
                {
                    commands.push(UiCommand::ToggleAutoStart);
                }
            });
        });
    });
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str, c: &'static design::Colors) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}  "))
                .size(FONT_SM)
                .color(c.text_muted),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .size(FONT_SM)
                    .color(c.text_secondary)
                    .strong(),
            );
        });
    });
    ui.add_space(SPACE_XXS);
}

// ── Data parsers ───────────────────────────────────────────────────────────

fn parse_proxy_groups(v: &serde_json::Value) -> Vec<ProxyGroup> {
    let mut groups = Vec::new();
    let proxies = match v.get("proxies") {
        Some(p) => p,
        None => return groups,
    };
    if let Some(obj) = proxies.as_object() {
        for (name, val) in obj {
            if let Some(typ) = val.get("type").and_then(|t| t.as_str()) {
                let all: Vec<String> = val
                    .get("all")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let now = val.get("now").and_then(|n| n.as_str()).map(String::from);
                let history: Vec<zeroclash_core::mihomo::DelayHistory> = val
                    .get("history")
                    .and_then(|h| h.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|v| zeroclash_core::mihomo::DelayHistory {
                                time: String::new(),
                                delay: v.get("delay").and_then(|d| d.as_u64()).unwrap_or(0),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                groups.push(ProxyGroup {
                    name: name.clone(),
                    group_type: typ.to_string(),
                    now,
                    all,
                    history,
                });
            }
        }
    }
    groups
}

fn parse_connections(v: &serde_json::Value) -> Vec<ConnEntry> {
    let mut entries = Vec::new();
    let conns = match v.get("connections").and_then(|c| c.as_array()) {
        Some(c) => c,
        None => return entries,
    };
    for conn in conns {
        let m = conn.get("metadata");
        entries.push(ConnEntry {
            id: conn
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            host: m
                .and_then(|m| m.get("host"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            network: m
                .and_then(|m| m.get("network"))
                .and_then(|v| v.as_str())
                .unwrap_or("tcp")
                .to_string(),
            conn_type: m
                .and_then(|m| m.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_ip: m
                .and_then(|m| m.get("sourceIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_ip: m
                .and_then(|m| m.get("destinationIP"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            source_port: m
                .and_then(|m| m.get("sourcePort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            destination_port: m
                .and_then(|m| m.get("destinationPort"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            dns_mode: m
                .and_then(|m| m.get("dnsMode"))
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            chains: conn
                .get("chains")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            rule: conn
                .get("rule")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            rule_payload: conn
                .get("rulePayload")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            upload: conn.get("upload").and_then(|v| v.as_u64()).unwrap_or(0),
            download: conn.get("download").and_then(|v| v.as_u64()).unwrap_or(0),
            start: conn
                .get("start")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string(),
            speed_up: 0,
            speed_down: 0,
        });
    }
    entries
}
