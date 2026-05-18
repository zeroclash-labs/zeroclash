use std::sync::Arc;

use egui_wgpu::wgpu;
use egui_winit::winit;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use zeroclash_core::mihomo::{CoreManager, ProxyGroup, Traffic};
use zeroclash_core::Config;

use crate::widgets::proxy_page::proxy_page_ui;
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

    // Application state
    config: Config,
    core_manager: Option<CoreManager>,
    core_running: bool,

    // Proxy data
    proxy_groups: Vec<ProxyGroup>,
    traffic_history: TrafficHistory,
    last_traffic: Traffic,

    // Page navigation
    current_page: Page,
    selected_proxy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Page {
    Home,
    Proxies,
    Settings,
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
            core_manager: None,
            core_running: false,
            proxy_groups: Vec::new(),
            traffic_history: TrafficHistory::default(),
            last_traffic: Traffic { up: 0, down: 0 },
            current_page: Page::Home,
            selected_proxy: None,
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
                let raw_input = state.egui_winit.take_egui_input(&state.window);

                // Clone what we need for the render closure
                let current_page = state.current_page.clone();
                let core_running = state.core_running;
                let proxy_groups = state.proxy_groups.clone();
                let traffic_history = std::mem::take(&mut state.traffic_history);
                let selected_proxy = state.selected_proxy.clone();

                let full_output = state.egui_ctx.run_ui(raw_input, |ui| {
                    render_ui(
                        ui,
                        &current_page,
                        core_running,
                        &proxy_groups,
                        &traffic_history,
                        selected_proxy.as_deref(),
                    );
                });

                // Restore traffic history
                state.traffic_history = traffic_history;

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

// ── UI rendering ───────────────────────────────────────────────────────────

fn render_ui(
    ui: &mut egui::Ui,
    current_page: &Page,
    core_running: bool,
    proxy_groups: &[ProxyGroup],
    traffic_history: &TrafficHistory,
    selected_proxy: Option<&str>,
) {
    // Sidebar
    egui::SidePanel::left("sidebar")
        .resizable(false)
        .default_width(180.0)
        .show_inside(ui, |ui| {
            ui.heading("ZeroClash");
            ui.separator();

            ui.vertical_centered(|ui| {
                let status_text = if core_running {
                    egui::RichText::new("● Core Running").color(egui::Color32::GREEN)
                } else {
                    egui::RichText::new("○ Core Stopped").color(egui::Color32::GRAY)
                };
                ui.label(status_text);
            });
            ui.separator();

            // Navigation
            nav_button(ui, "Home", Page::Home, current_page);
            nav_button(ui, "Proxies", Page::Proxies, current_page);
            nav_button(ui, "Settings", Page::Settings, current_page);
        });

    // Main content area
    egui::CentralPanel::default().show_inside(ui, |ui| {
        match current_page {
            Page::Home => home_page_ui(ui, core_running, traffic_history),
            Page::Proxies => {
                proxy_page_ui(ui, proxy_groups, None, &|group, proxy| {
                    log::info!("Select proxy {proxy} in group {group}");
                });
            }
            Page::Settings => settings_page_ui(ui),
        }
    });
}

fn nav_button(ui: &mut egui::Ui, label: &str, page: Page, current: &Page) {
    let selected = *current == page;
    if ui
        .selectable_label(selected, label)
        .clicked()
    {
        // Navigation handled by parent state
        // In a real app we'd send a channel message here
    }
}

fn home_page_ui(ui: &mut egui::Ui, core_running: bool, traffic: &TrafficHistory) {
    ui.heading("Dashboard");

    // Traffic summary
    traffic_summary_ui(ui, traffic);
    ui.separator();

    // Core status card
    egui::Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(egui::vec2(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Core Status").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if core_running {
                        if ui.button("Stop Core").clicked() {}
                    } else if ui.button("Start Core").clicked() {}
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("HTTP Proxy:");
                ui.label("127.0.0.1:7899");
            });
            ui.horizontal(|ui| {
                ui.label("SOCKS Proxy:");
                ui.label("127.0.0.1:7898");
            });
            ui.horizontal(|ui| {
                ui.label("Mixed Port:");
                ui.label("127.0.0.1:7897");
            });
        });

    ui.add_space(12.0);

    // System info
    egui::Frame::default()
        .corner_radius(6)
        .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(egui::vec2(12.0, 8.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("System").strong());
            ui.separator();
            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
        });
}

fn settings_page_ui(ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.label("Configuration options will be available in a future update.");
}
