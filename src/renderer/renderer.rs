use egui_wgpu::ScreenDescriptor;
use log::{debug, warn};
use std::{collections::HashMap, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::{
    gui_renderer::GuiRenderer,
    pass::DefaultPass,
    renderable::{Renderable, Vertex},
};

pub struct Renderer {
    _instance: Instance,
    pub window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pub gui_renderer: GuiRenderer,

    renderables: HashMap<String, Renderable>,
    passes: HashMap<String, DefaultPass>,
    global_buffers: HashMap<String, Buffer>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let renderables = HashMap::new();
        let passes = HashMap::new();
        let buffers = HashMap::new();

        let backends = wgpu::Backends::PRIMARY;
        let instance = Instance::new(InstanceDescriptor {
            backends,
            ..Default::default()
        });
        debug!("Instance created");
        let surface = instance.create_surface(window.clone()).unwrap();
        debug!("Surface created");
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug!("Adapter acquired");
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        debug!("Device and Queue acquired");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|s| s.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2, // max images in flight for swapchain
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        debug!("Surface configured");

        // let default_pass = DefaultPass::new(&device, &config);
        // passes.insert("default".into(), default_pass);

        let gui_renderer = GuiRenderer::new(&device, surface_format, None, 1, &window);

        debug!("Renderer initialized");
        Self {
            _instance: instance,
            window,
            device,
            queue,
            surface,
            config,
            size,

            gui_renderer,

            renderables,
            passes,
            global_buffers: buffers,
        }
    }

    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn add_global_buffer(&mut self, name: String, data: &[u8], usage: BufferUsages) {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(name.as_str()),
            contents: data,
            usage,
        });
        self.global_buffers.insert(name, buffer);
    }
    pub fn write_buffer(&mut self, name: &str, data: &[u8]) {
        if let Some(buffer) = self.global_buffers.get(name) {
            self.queue.write_buffer(buffer, 0, data);
        }
    }

    pub fn add_pass(&mut self, name: String) {
        let pass = DefaultPass::new(&self.device, &self.config, &self.global_buffers);
        self.passes.insert(name, pass);
    }

    pub fn add_renderable(
        &mut self,
        name: String,
        pipeline_tag: String,
        indices: &[u16],
        vertices: &[Vertex],
    ) {
        debug!(
            "New renderable {} added, using pipeline {}",
            name, pipeline_tag
        );
        self.renderables.insert(
            name,
            Renderable::new(&self.device, vertices, indices, pipeline_tag),
        );
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            debug!("Resized to {}x{}", new_size.width, new_size.height);
        }
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        // main pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main renderpass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // TODO: add depth
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // issue drawcalls
            for pass_name in self.passes.keys() {
                let renderables = self
                    .renderables
                    .values()
                    .filter(|x| &x.pass_name == pass_name);
                if let Some(pass) = self.passes.get(pass_name) {
                    // debug!("drawing {} renderables with pass {}", renderables.collect::<Vec<_>>().len(), pass_name);
                    pass.draw(&mut render_pass, renderables)?;
                } else {
                    warn!("pass with name {} does not exist!", pass_name);
                }
            }
        }
        // gui pass
        let screen_desc = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        self.gui_renderer.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &self.window,
            screen_desc,
        );

        self.queue.submit([encoder.finish()]);
        output.present();
        Ok(())
    }
}
