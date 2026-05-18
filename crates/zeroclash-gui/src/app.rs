use std::path::PathBuf;
use std::sync::Arc;

use egui_wgpu::wgpu;
use egui_winit::winit;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use zeroclash_core::config::VergeConfig;
use zeroclash_core::mihomo::{CoreManager, ProxyGroup};
use zeroclash_core::profile::ProfilePreview;
use zeroclash_core::{Config, ProfileStore};

use crate::widgets::profile_page::{ImportDialog, profile_page_ui};
use crate::widgets::proxy_page::proxy_page_ui;
use crate::widgets::settings_page::settings_page_ui;
use crate::widgets::traffic_graph::{TrafficHistory, traffic_summary_ui};

// ── Application ────────────────────────────────────────────────────────────

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Home,
    Proxies,
    Profiles,
    Settings,
}

enum UiCommand {
    ActivateProfile(String),
    DeleteProfile(String),
    ImportProfile(String),
    SaveConfig(VergeConfig),
    ToggleCore,
    Navigate(Page),
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

        let window_attrs = Window::default_attributes()
            .with_title("ZeroClash")
            .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0));

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create wgpu surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find wgpu adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
        ))
        .expect("Failed to create wgpu device");

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );

        let egui_renderer =
            egui_wgpu::Renderer::new(&device, surface_format, egui_wgpu::RendererOptions::default());

        self.state = Some(AppState {
            window,
            egui_ctx,
            egui_winit,
            egui_renderer,
            wgpu_surface: surface,
            wgpu_device: device,
            wgpu_queue: queue,
            wgpu_config: config,
            config: Config::new(),
            profile_store: None,
            core_manager: None,
            core_running: false,
            proxy_groups: Vec::new(),
            traffic_history: TrafficHistory::default(),
            current_page: Page::Home,
            import_dialog: ImportDialog::new(),
            pending_commands: Vec::new(),
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };

        let _response = state.egui_winit.on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let commands = std::mem::take(&mut state.pending_commands);
                for cmd in commands {
                    process_command(state, cmd);
                }

                let raw_input = state.egui_winit.take_egui_input(&state.window);

                // Clone what we need for rendering (avoids double-borrow of state)
                let core_running = state.core_running;
                let current_page = state.current_page.clone();
                let traffic = TrafficHistory::default(); // placeholder
                let verge = state.config.verge.latest_arc().as_ref().clone();
                let previews = state.profile_store.as_ref().map(|ps| ps.preview()).unwrap_or_default();

                let full_output = state.egui_ctx.run_ui(raw_input, |ui| {
                    render_ui(
                        ui,
                        &current_page,
                        core_running,
                        &verge,
                        &previews,
                        &mut state.import_dialog,
                        &mut state.pending_commands,
                    );
                });

                state
                    .egui_winit
                    .handle_platform_output(&state.window, full_output.platform_output);

                let paint_jobs = state
                    .egui_ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                for (id, delta) in &full_output.textures_delta.set {
                    state
                        .egui_renderer
                        .update_texture(&state.wgpu_device, &state.wgpu_queue, *id, delta);
                }

                let surface_texture = match state.wgpu_surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) => t,
                    wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => {
                        state.window.request_redraw();
                        return;
                    }
                };

                let output_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = state.wgpu_device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { label: None },
                );

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [state.wgpu_config.width, state.wgpu_config.height],
                    pixels_per_point: state.window.scale_factor() as f32,
                };

                state.egui_renderer.update_buffers(
                    &state.wgpu_device,
                    &state.wgpu_queue,
                    &mut encoder,
                    &paint_jobs,
                    &screen_descriptor,
                );

                let mut render_pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &output_view,
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

                state
                    .egui_renderer
                    .render(&mut render_pass, &paint_jobs, &screen_descriptor);

                drop(render_pass);

                state.wgpu_queue.submit(std::iter::once(encoder.finish()));
                surface_texture.present();

                for id in &full_output.textures_delta.free {
                    state.egui_renderer.free_texture(id);
                }

                state.window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                state.wgpu_config.width = new_size.width;
                state.wgpu_config.height = new_size.height;
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
        UiCommand::Navigate(page) => {
            state.current_page = page;
        }
        UiCommand::ActivateProfile(uid) => {
            if let Some(ref mut store) = state.profile_store {
                let _ = store.set_current(&uid);
                let _ = pollster::block_on(store.save());
            }
        }
        UiCommand::DeleteProfile(uid) => {
            if let Some(ref mut store) = state.profile_store {
                let _ = store.delete_item(&uid);
                let _ = pollster::block_on(store.save());
            }
        }
        UiCommand::ImportProfile(url) => {
            if state.profile_store.is_none() {
                let data_dir = dirs_next::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("zeroclash");
                let _ = std::fs::create_dir_all(&data_dir);
                state.profile_store = pollster::block_on(ProfileStore::load(data_dir)).ok();
            }

            if let Some(ref store) = state.profile_store {
                match pollster::block_on(store.fetch_remote(&url, None, None)) {
                    Ok(item) => {
                        let mut store_mut = state.profile_store.take().unwrap();
                        let _ = store_mut.add_item(item);
                        let _ = pollster::block_on(store_mut.save());
                        state.profile_store = Some(store_mut);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch profile: {e}");
                    }
                }
            }
        }
        UiCommand::SaveConfig(verge) => {
            state.config.verge.edit_draft(|v| {
                *v = verge;
            });
            state.config.verge.apply();
        }
        UiCommand::ToggleCore => {
            state.core_running = !state.core_running;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_ui(
    ui: &mut egui::Ui,
    current_page: &Page,
    core_running: bool,
    verge: &VergeConfig,
    previews: &[ProfilePreview],
    import_dialog: &mut ImportDialog,
    commands: &mut Vec<UiCommand>,
) {
    // Sidebar
    egui::Panel::left("sidebar")
        .resizable(false)
        .default_size(180.0)
        .show_inside(ui, |ui| {
            ui.heading("ZeroClash");
            ui.separator();

            ui.vertical_centered(|ui| {
                let status = if core_running {
                    egui::RichText::new("● Core Running").color(egui::Color32::GREEN)
                } else {
                    egui::RichText::new("○ Core Stopped").color(egui::Color32::GRAY)
                };
                ui.label(status);
            });
            ui.separator();

            for (label, page) in [
                ("Home", Page::Home),
                ("Proxies", Page::Proxies),
                ("Profiles", Page::Profiles),
                ("Settings", Page::Settings),
            ] {
                if ui
                    .selectable_label(current_page == &page, label)
                    .clicked()
                {
                    commands.push(UiCommand::Navigate(page));
                }
            }
        });

    // Main content
    egui::CentralPanel::default().show_inside(ui, |ui| {
        match current_page {
            Page::Home => home_page_ui(ui, core_running, &TrafficHistory::default(), commands),
            Page::Proxies => {
                proxy_page_ui(ui, &[], None, &|group, proxy| {
                    log::info!("Select proxy {proxy} in group {group}");
                });
            }
            Page::Profiles => {
                use std::cell::RefCell;
                let activate = RefCell::new(String::new());
                let delete = RefCell::new(String::new());
                let import = RefCell::new(String::new());

                profile_page_ui(
                    ui,
                    previews,
                    import_dialog,
                    |uid| *activate.borrow_mut() = uid.to_string(),
                    |uid| *delete.borrow_mut() = uid.to_string(),
                    |url| *import.borrow_mut() = url.to_string(),
                );

                let a = activate.borrow().clone();
                let d = delete.borrow().clone();
                let i = import.borrow().clone();
                if !a.is_empty() {
                    commands.push(UiCommand::ActivateProfile(a));
                }
                if !d.is_empty() {
                    commands.push(UiCommand::DeleteProfile(d));
                }
                if !i.is_empty() {
                    commands.push(UiCommand::ImportProfile(i));
                }
            }
            Page::Settings => {
                let mut v = verge.clone();
                settings_page_ui(ui, &mut v, &mut |cfg| {
                    commands.push(UiCommand::SaveConfig(cfg.clone()));
                });
            }
        }
    });
}

fn home_page_ui(
    ui: &mut egui::Ui,
    core_running: bool,
    traffic: &TrafficHistory,
    commands: &mut Vec<UiCommand>,
) {
    ui.heading("Dashboard");

    traffic_summary_ui(ui, traffic);
    ui.separator();

    egui::Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(egui::vec2(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Core Status").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if core_running {
                        if ui.button("Stop Core").clicked() {
                            commands.push(UiCommand::ToggleCore);
                        }
                    } else if ui.button("Start Core").clicked() {
                        commands.push(UiCommand::ToggleCore);
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("HTTP Proxy:"); ui.label("127.0.0.1:7899");
            });
            ui.horizontal(|ui| {
                ui.label("SOCKS Proxy:"); ui.label("127.0.0.1:7898");
            });
            ui.horizontal(|ui| {
                ui.label("Mixed Port:"); ui.label("127.0.0.1:7897");
            });
        });

    ui.add_space(12.0);
    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
}
