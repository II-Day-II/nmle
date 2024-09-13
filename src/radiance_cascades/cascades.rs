use vek::Vec2;
use wgpu::{
    include_spirv, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Color, ColorTargetState, ColorWrites, CommandEncoder, Device,
    FragmentState, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerDescriptor, ShaderStages, TextureFormat, TextureView, VertexState,
};

use crate::renderer::include_shader;
pub struct RadianceCascades {
    start_interval: f32,
    cascade_count: u8,

    gi_pipeline: RenderPipeline,
    gi_bg_layout: BindGroupLayout,

    sampler: Sampler,
}

impl RadianceCascades {
    pub fn new(device: &Device, screen_size: Vec2<f32>) -> Self {
        let fs_tri_module =
            device.create_shader_module(include_shader!("/shaders/fullscreen_tri.vert.spv"));
        let gi_module = device.create_shader_module(include_shader!("/shaders/gi.frag.spv"));

        let gi_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("gi bg layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let gi_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("gi_pipeline_layout"),
            bind_group_layouts: &[&gi_bg_layout],
            push_constant_ranges: &[],
        });
        let gi_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("gi_pipeline"),
            layout: Some(&gi_pipeline_layout),
            vertex: VertexState {
                module: &fs_tri_module,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &gi_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb, // TODO: fix this,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("gi_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mut rc = Self {
            start_interval: 0.0,
            cascade_count: 0,
            gi_pipeline,
            gi_bg_layout,
            sampler,
        };
        rc.resize(screen_size);
        rc
    }

    pub fn resize(&mut self, screen_size: Vec2<f32>) {
        let branching_factor = 2.0f32;
        let interval0 = 4.0; // TODO: what should this be?
        let diagonal = screen_size.distance(Vec2::new(0.0, 0.0)); // no length()?
        let factor = (diagonal / interval0).log(branching_factor).ceil();
        let start_interval = (interval0 * branching_factor.powf(factor)) / (branching_factor - 1.0);
        let cascade_count = start_interval.log(branching_factor).ceil() as u8;
        self.cascade_count = cascade_count;
        self.start_interval = start_interval;
    }

    pub fn run(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        input_render_view: &TextureView,
        input_distance_view: &TextureView,
        output_tex_view: &TextureView,
    ) {
        let gi_bindgroup = device.create_bind_group(&BindGroupDescriptor {
            label: Some("gi_bindgroup"),
            layout: &self.gi_bg_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_render_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&input_distance_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("GI pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output_tex_view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.gi_pipeline);
        pass.set_bind_group(0, &gi_bindgroup, &[]);
        pass.draw(0..3, 0..1);
    }
}
