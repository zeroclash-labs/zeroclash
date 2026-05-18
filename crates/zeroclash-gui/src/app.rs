use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use egui_wgpu::wgpu;
use egui_winit::winit;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use zeroclash_core::config::VergeConfig;
use zeroclash_core::connection::ConnEntry;
use zeroclash_core::mihomo::{CoreManager, ProxyGroup};
use zeroclash_core::profile::ProfilePreview;
use zeroclash_core::{Config, ProfileStore, SystemProxy, acquire_singleton, notify};

use crate::tray::{SystemTray, TrayEvent};

use crate::widgets::connection_table::connection_table_ui;
use crate::widgets::log_viewer::{LogLevel, LogViewer, log_viewer_ui};
use crate::widgets::profile_page::{ImportDialog, profile_page_ui};
use crate::widgets::proxy_page::proxy_page_ui;
use crate::widgets::settings_page::settings_page_ui;
use crate::widgets::traffic_graph::{TrafficHistory, traffic_summary_ui};

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
    core_manager: Option<CoreManager>,
    core_running: bool,
    proxy_groups: Vec<ProxyGroup>,
    traffic_history: TrafficHistory,
    current_page: Page,
    import_dialog: ImportDialog,
    pending_commands: Vec<UiCommand>,
    // Connection & log state
    connections: Vec<ConnEntry>,
    selected_conn_id: Option<String>,
    log_viewer: LogViewer,
    // System integration
    _tray: Option<SystemTray>,
    window_visible: Arc<AtomicBool>,
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        // ... window/surface setup identical to Phase 2 ...
        let window_attrs = Window::default_attributes()
            .with_title("ZeroClash")
            .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0));
        let window = Arc::new(event_loop.create_window(window_attrs).expect("window"));

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone()).expect("surface");
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
        let fmt = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);
        let cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: fmt, width: size.width, height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![], desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &cfg);
        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &*window, None, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, fmt, egui_wgpu::RendererOptions::default());

        let mut log_viewer = LogViewer::default();
        log_viewer.push(LogLevel::Info, "zeroclash", "Application started");

        // Singleton check
        match acquire_singleton("zeroclash") {
            Ok(true) => log_viewer.push(LogLevel::Info, "sys", "First instance, acquired singleton lock"),
            Ok(false) => log_viewer.push(LogLevel::Warn, "sys", "Another instance may be running"),
            Err(e) => log_viewer.push(LogLevel::Error, "sys", &format!("Singleton check failed: {e}")),
        }

        // System tray
        let window_visible = Arc::new(AtomicBool::new(true));
        let tray = SystemTray::new(window_visible.clone())
            .map_err(|e| log_viewer.push(LogLevel::Warn, "tray", &format!("Tray creation failed: {e}")))
            .ok();

        // Welcome notification
        notify("ZeroClash", "Application started successfully");

        self.state = Some(AppState {
            window, egui_ctx, egui_winit, egui_renderer,
            wgpu_surface: surface, wgpu_device: device, wgpu_queue: queue,
            wgpu_config: cfg,
            config: Config::new(),
            profile_store: None, core_manager: None, core_running: false,
            proxy_groups: Vec::new(), traffic_history: TrafficHistory::default(),
            current_page: Page::Home, import_dialog: ImportDialog::new(),
            pending_commands: Vec::new(),
            connections: Vec::new(), selected_conn_id: None, log_viewer,
            _tray: tray, window_visible,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state { Some(s) => s, None => return };
        let _ = state.egui_winit.on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let commands = std::mem::take(&mut state.pending_commands);
                for cmd in commands { process_command(state, cmd); }

                let raw_input = state.egui_winit.take_egui_input(&state.window);
                let current_page = state.current_page.clone();
                let core_running = state.core_running;
                let verge = state.config.verge.latest_arc().as_ref().clone();
                let previews = state.profile_store.as_ref().map(|ps| ps.preview()).unwrap_or_default();

                let full_output = state.egui_ctx.run_ui(raw_input, |ui| {
                    render_ui(ui, &current_page, core_running, &verge, &previews,
                        &mut state.import_dialog, &mut state.pending_commands,
                        &state.connections, &mut state.selected_conn_id,
                        &mut state.log_viewer);
                });

                state.egui_winit.handle_platform_output(&state.window, full_output.platform_output);
                let paint_jobs = state.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                for (id, d) in &full_output.textures_delta.set {
                    state.egui_renderer.update_texture(&state.wgpu_device, &state.wgpu_queue, *id, d);
                }
                let surface_texture = match state.wgpu_surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => { state.window.request_redraw(); return; }
                };
                let output_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state.wgpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let sd = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [state.wgpu_config.width, state.wgpu_config.height],
                    pixels_per_point: state.window.scale_factor() as f32,
                };
                state.egui_renderer.update_buffers(&state.wgpu_device, &state.wgpu_queue, &mut encoder, &paint_jobs, &sd);
                let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view, depth_slice: None, resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None, timestamp_writes: None,
                    occlusion_query_set: None, multiview_mask: None,
                }).forget_lifetime();
                state.egui_renderer.render(&mut rp, &paint_jobs, &sd);
                drop(rp);
                state.wgpu_queue.submit(std::iter::once(encoder.finish()));
                surface_texture.present();
                for id in &full_output.textures_delta.free { state.egui_renderer.free_texture(id); }
                state.window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                state.wgpu_config.width = new_size.width;
                state.wgpu_config.height = new_size.height;
                state.wgpu_surface.configure(&state.wgpu_device, &state.wgpu_config);
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
            if let Some(ref mut s) = state.profile_store { let _ = s.set_current(&uid); let _ = pollster::block_on(s.save()); }
            state.log_viewer.push(LogLevel::Info, "profile", &format!("Activated profile {uid}"));
        }
        UiCommand::DeleteProfile(uid) => {
            if let Some(ref mut s) = state.profile_store { let _ = s.delete_item(&uid); let _ = pollster::block_on(s.save()); }
            state.log_viewer.push(LogLevel::Info, "profile", &format!("Deleted profile {uid}"));
        }
        UiCommand::ImportProfile(url) => {
            if state.profile_store.is_none() {
                let dir = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("zeroclash");
                let _ = std::fs::create_dir_all(&dir);
                state.profile_store = pollster::block_on(ProfileStore::load(dir)).ok();
            }
            if let Some(ref store) = state.profile_store {
                match pollster::block_on(store.fetch_remote(&url, None, None)) {
                    Ok(item) => {
                        let name = item.name.clone().unwrap_or_default();
                        let mut sm = state.profile_store.take().unwrap();
                        let _ = sm.add_item(item); let _ = pollster::block_on(sm.save());
                        state.profile_store = Some(sm);
                        state.log_viewer.push(LogLevel::Info, "profile", &format!("Imported profile {name}"));
                    }
                    Err(e) => state.log_viewer.push(LogLevel::Error, "profile", &format!("Import failed: {e}")),
                }
            }
        }
        UiCommand::SaveConfig(v) => { state.config.verge.edit_draft(|c| *c = v); state.config.verge.apply(); }
        UiCommand::ToggleCore => { state.core_running = !state.core_running; }
        UiCommand::CloseConnection(id) => {
            state.log_viewer.push(LogLevel::Info, "conn", &format!("Closed connection {id}"));
            if state.selected_conn_id.as_deref() == Some(&id) { state.selected_conn_id = None; }
        }
        UiCommand::ToggleSystemProxy => {
            let enabled = state.config.verge.latest_arc().enable_system_proxy;
            if !enabled {
                match SystemProxy::enable(7899, 7898) {
                    Ok(()) => {
                        state.config.verge.edit_draft(|c| c.enable_system_proxy = true);
                        state.config.verge.apply();
                        notify("ZeroClash", "System proxy enabled");
                        state.log_viewer.push(LogLevel::Info, "sys", "System proxy enabled");
                    }
                    Err(e) => state.log_viewer.push(LogLevel::Error, "sys", &format!("Proxy enable failed: {e}")),
                }
            } else {
                match SystemProxy::disable() {
                    Ok(()) => {
                        state.config.verge.edit_draft(|c| c.enable_system_proxy = false);
                        state.config.verge.apply();
                        notify("ZeroClash", "System proxy disabled");
                        state.log_viewer.push(LogLevel::Info, "sys", "System proxy disabled");
                    }
                    Err(e) => state.log_viewer.push(LogLevel::Error, "sys", &format!("Proxy disable failed: {e}")),
                }
            }
        }
        UiCommand::ToggleAutoStart => {
            let auto = zeroclash_core::AutoStart::new("zeroclash", std::env::current_exe().unwrap_or_default());
            if auto.is_enabled() {
                match auto.disable() {
                    Ok(()) => {
                        state.config.verge.edit_draft(|c| c.enable_auto_launch = false);
                        state.config.verge.apply();
                        state.log_viewer.push(LogLevel::Info, "sys", "Auto-start disabled");
                    }
                    Err(e) => state.log_viewer.push(LogLevel::Error, "sys", &format!("Auto-start disable failed: {e}")),
                }
            } else {
                match auto.enable() {
                    Ok(()) => {
                        state.config.verge.edit_draft(|c| c.enable_auto_launch = true);
                        state.config.verge.apply();
                        state.log_viewer.push(LogLevel::Info, "sys", "Auto-start enabled");
                    }
                    Err(e) => state.log_viewer.push(LogLevel::Error, "sys", &format!("Auto-start enable failed: {e}")),
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_ui(
    ui: &mut egui::Ui, current_page: &Page, core_running: bool,
    verge: &VergeConfig, previews: &[ProfilePreview],
    import_dialog: &mut ImportDialog, commands: &mut Vec<UiCommand>,
    connections: &[ConnEntry], selected_conn_id: &mut Option<String>,
    log_viewer: &mut LogViewer,
) {
    egui::Panel::left("sidebar").resizable(false).default_size(180.0).show_inside(ui, |ui| {
        ui.heading("ZeroClash"); ui.separator();
        ui.vertical_centered(|ui| {
            let s = if core_running { egui::RichText::new("● Core Running").color(egui::Color32::GREEN) }
            else { egui::RichText::new("○ Core Stopped").color(egui::Color32::GRAY) };
            ui.label(s);
        });
        ui.separator();
        for (label, page) in [
            ("Home", Page::Home), ("Proxies", Page::Proxies), ("Profiles", Page::Profiles),
            ("Connections", Page::Connections), ("Logs", Page::Logs), ("Settings", Page::Settings),
        ] {
            if ui.selectable_label(current_page == &page, label).clicked() {
                commands.push(UiCommand::Navigate(page));
            }
        }
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        match current_page {
            Page::Home => home_page_ui(ui, core_running, &TrafficHistory::default(), commands),
            Page::Proxies => proxy_page_ui(ui, &[], None, &|g, p| log::info!("Select {p} in {g}")),
            Page::Profiles => {
                use std::cell::RefCell;
                let (a, d, i) = (RefCell::new(String::new()), RefCell::new(String::new()), RefCell::new(String::new()));
                profile_page_ui(ui, previews, import_dialog,
                    |uid| *a.borrow_mut() = uid.to_string(),
                    |uid| *d.borrow_mut() = uid.to_string(),
                    |url| *i.borrow_mut() = url.to_string());
                let (av, dv, iv) = (a.borrow().clone(), d.borrow().clone(), i.borrow().clone());
                if !av.is_empty() { commands.push(UiCommand::ActivateProfile(av)); }
                if !dv.is_empty() { commands.push(UiCommand::DeleteProfile(dv)); }
                if !iv.is_empty() { commands.push(UiCommand::ImportProfile(iv)); }
            }
            Page::Connections => connection_table_ui(ui, connections, selected_conn_id,
                |id| commands.push(UiCommand::CloseConnection(id.to_string()))),
            Page::Logs => log_viewer_ui(ui, log_viewer),
            Page::Settings => {
                let mut v = verge.clone();
                settings_page_ui(ui, &mut v, &mut |cfg| commands.push(UiCommand::SaveConfig(cfg.clone())));
            }
        }
    });
}

fn home_page_ui(ui: &mut egui::Ui, core_running: bool, traffic: &TrafficHistory, commands: &mut Vec<UiCommand>) {
    ui.heading("Dashboard");
    traffic_summary_ui(ui, traffic);
    ui.separator();
    egui::Frame::default().corner_radius(6).stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(egui::vec2(12.0, 8.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Core Status").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if core_running { if ui.button("Stop Core").clicked() { commands.push(UiCommand::ToggleCore); } }
                    else if ui.button("Start Core").clicked() { commands.push(UiCommand::ToggleCore); }
                });
            });
            ui.separator();
            ui.horizontal(|ui| { ui.label("HTTP:"); ui.label("127.0.0.1:7899"); });
            ui.horizontal(|ui| { ui.label("SOCKS:"); ui.label("127.0.0.1:7898"); });
        });
    ui.add_space(12.0);
    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
}
