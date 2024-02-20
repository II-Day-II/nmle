use log::debug;
use std::{collections::HashMap, sync::Arc};
use wgpu::{
    include_spirv, BlendState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Instance, InstanceDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::{
    pipeline::{Pipeline, PipelineBuilder},
    renderable::{Renderable, Vertex},
};

macro_rules! include_shader {
    ($file:expr) => {
        include_spirv!(concat!(env!("OUT_DIR"), $file))
    };
}

pub struct Renderer {
    _instance: Instance,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    pipelines: HashMap<String, Pipeline>,
    renderables: HashMap<String, Renderable>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let backends = wgpu::Backends::PRIMARY;
        let instance = Instance::new(InstanceDescriptor {
            backends,
            ..Default::default()
        });
        debug!("Instance created");
        // safety: We own the window, and it is 'static. it lives long enough
        let surface = instance.create_surface(window).unwrap();
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
            .filter(|s| s.is_srgb())
            .next()
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

        // TODO: see if this works in web
        let v_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.vert.spv"));
        let f_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.frag.spv"));
        let mut builder = PipelineBuilder::new(&v_shader_spv, Some("main"));
        let default_pipeline = builder
            .with_fragment_shader(&f_shader_spv, Some("main"))
            .with_cull_mode(wgpu::Face::Back)
            .add_fragment_target(Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            }))
            .build(&device);

        let mut pipelines = HashMap::new();
        pipelines.insert("default".into(), default_pipeline);
        debug!("Default pipeline constructed");

        let renderables = HashMap::new();

        debug!("Renderer initialized");
        Self {
            _instance: instance,
            device,
            queue,
            surface,
            config,
            size,

            pipelines,
            renderables,
        }
    }

    pub fn _add_pipeline(&mut self, pipeline_builder: &mut PipelineBuilder, pipeline_tag: String) {
        let pipeline = pipeline_builder.build(&self.device);
        debug!("New pipeline {} added", pipeline_tag);
        self.pipelines.insert(pipeline_tag, pipeline);
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
            for renderable in self.renderables.values() {
                let pipeline = &self.pipelines[&renderable.pipeline_tag];
                render_pass.set_pipeline(&pipeline.pipeline);
                renderable.draw(&mut render_pass);
            }
        }
        self.queue.submit([encoder.finish()]);
        output.present();
        Ok(())
    }
}
