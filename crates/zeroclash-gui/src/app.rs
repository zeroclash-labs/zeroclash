use std::sync::Arc;

use egui_wgpu::wgpu;
use egui_winit::winit;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use zeroclash_core::Config;

/// The main ZeroClash application.
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
    core_running: bool,
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

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );

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
            core_running: false,
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
                let full_output = state.egui_ctx.run_ui(raw_input, |ui| {
                    render_ui(ui, &state.config, &mut state.core_running);
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

                // wgpu 29: get_current_texture returns CurrentSurfaceTexture enum
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

fn render_ui(ui: &mut egui::Ui, _config: &Config, core_running: &mut bool) {
    ui.heading("ZeroClash");
    ui.separator();

    ui.horizontal(|ui| {
        if *core_running {
            ui.label("Core: Running");
            if ui.button("Stop Core").clicked() {
                *core_running = false;
            }
        } else {
            ui.label("Core: Stopped");
            if ui.button("Start Core").clicked() {
                *core_running = true;
            }
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Status:");
        ui.label("v0.0.1");
    });

    ui.horizontal(|ui| {
        ui.label("HTTP Port:");
        ui.label("7899");
    });

    ui.horizontal(|ui| {
        ui.label("SOCKS Port:");
        ui.label("7898");
    });
}
