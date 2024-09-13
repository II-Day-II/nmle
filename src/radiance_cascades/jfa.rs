use std::mem::swap;

use log::debug;
use vek::Vec2;
use wgpu::{
    include_spirv,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d, FragmentState,
    MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerDescriptor, ShaderStages, Texture, TextureDescriptor, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexState,
};

use crate::renderer::include_shader;

struct PingPongTex {
    texture: Texture,
    view: TextureView,
}
impl PingPongTex {
    pub fn new(device: &Device, size: Vec2<u32>, label: &str) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Bgra8Unorm],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self { texture, view }
    }
}
pub struct JumpFlood {
    pub num_passes: u32,
    ping_pong_a: PingPongTex,
    ping_pong_b: PingPongTex,

    seed_pipeline: RenderPipeline,
    seed_bindgroup_layout: BindGroupLayout,

    jfa_pipeline: RenderPipeline,
    jfa_bindgroup_layout: BindGroupLayout,

    distance_field_pipeline: RenderPipeline,
    distance_field_bg_layout: BindGroupLayout,

    sampler: Sampler,
}

impl JumpFlood {
    pub fn new(device: &Device, screen_size: Vec2<f32>) -> Self {
        let num_passes = screen_size.reduce_partial_max().log2().ceil() as u32;
        let ping_pong_a = PingPongTex::new(
            &device,
            Vec2::new(screen_size.x as u32, screen_size.y as u32),
            "PingA",
        );
        let ping_pong_b = PingPongTex::new(
            &device,
            Vec2::new(screen_size.x as u32, screen_size.y as u32),
            "PongB",
        );
        debug!("pingpong setup done");
        let fullscreen_tri_module =
            device.create_shader_module(include_shader!("/shaders/fullscreen_tri.vert.spv"));
        debug!("fullscreen tri shader loaded");
        let jfa_seed_module =
            device.create_shader_module(include_shader!("/shaders/jfa_seed.frag.spv"));
        debug!("jfa seed shader loaded");
        let jfa_module = device.create_shader_module(include_shader!("/shaders/jfa.frag.spv"));
        debug!("jfa shader loaded");
        let df_module =
            device.create_shader_module(include_shader!("/shaders/distance_field.frag.spv"));

        let seed_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("SeedBgLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let seed_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Seed pipeline layout"),
            bind_group_layouts: &[&seed_bindgroup_layout],
            push_constant_ranges: &[],
        });
        let seed_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Seed pipeline"),
            layout: Some(&seed_pipeline_layout),
            vertex: VertexState {
                module: &fullscreen_tri_module,
                entry_point: "main",
                buffers: &[], // not using a vertex buffer for this, so [] should be fine?
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &jfa_seed_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: ping_pong_a.texture.format(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let jfa_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("JFA bg layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let jfa_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("JFA pipeline layout"),
            bind_group_layouts: &[&jfa_bindgroup_layout],
            push_constant_ranges: &[],
        });
        let jfa_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("JFA pipeline"),
            layout: Some(&jfa_pipeline_layout),
            vertex: VertexState {
                module: &fullscreen_tri_module,
                entry_point: "main",
                buffers: &[], // not using a vertex buffer for this, so [] should be fine?
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &jfa_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: ping_pong_b.texture.format(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let distance_field_bg_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("distance field bg layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let df_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("distance field pipeline layout"),
            bind_group_layouts: &[&distance_field_bg_layout],
            push_constant_ranges: &[],
        });
        let distance_field_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("DF pipeline"),
            layout: Some(&df_pipeline_layout),
            vertex: VertexState {
                module: &fullscreen_tri_module,
                entry_point: "main",
                buffers: &[], // not using a vertex buffer for this, so [] should be fine?
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &df_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    // format: ping_pong_b.texture.format(),
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("JFA sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            num_passes,
            ping_pong_a,
            ping_pong_b,

            seed_pipeline,
            seed_bindgroup_layout,

            jfa_pipeline,
            jfa_bindgroup_layout,

            distance_field_bg_layout,
            distance_field_pipeline,

            sampler,
        }
    }
    pub fn resize(&mut self, device: &Device, screen_size: Vec2<f32>) {
        self.num_passes = screen_size.reduce_partial_max().log2().ceil() as u32;

        self.ping_pong_a = PingPongTex::new(
            &device,
            Vec2::new(screen_size.x as u32, screen_size.y as u32),
            "PingA",
        );
        self.ping_pong_b = PingPongTex::new(
            &device,
            Vec2::new(screen_size.x as u32, screen_size.y as u32),
            "PongB",
        );
    }

    // run jfa to create a distance field to the non transparent pixels in input texture,
    // stored in output texture
    pub fn run(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        input_texture_view: &TextureView,
        output_texture_view: &TextureView,
    ) {
        let mut current_output_texture = &mut self.ping_pong_a;
        let mut current_input_texture = &mut self.ping_pong_b;
        // create seed texture
        {
            let seed_bindgroup = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Seed bindgroup"),
                layout: &self.seed_bindgroup_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(input_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut seed_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Seed pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &current_output_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            seed_pass.set_pipeline(&self.seed_pipeline);
            seed_pass.set_bind_group(0, &seed_bindgroup, &[]);
            seed_pass.draw(0..3, 0..1);
        }

        // ping-pong the jfa textures
        //for i in 0..2*9 {
        for i in 0..self.num_passes {
            swap(&mut current_output_texture, &mut current_input_texture);
            let offset = 2.0f32.powi((self.num_passes - i - 1) as i32);
            // let offset = 2.0f32.powi(2*9 - i - 1);
            let jfa_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("jfa uniform buffer"),
                contents: bytemuck::bytes_of(&offset),
                usage: BufferUsages::UNIFORM,
            });

            let jfa_bindgroup = device.create_bind_group(&BindGroupDescriptor {
                label: Some("jfa bindgroup"),
                layout: &self.jfa_bindgroup_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(BufferBinding {
                            buffer: &jfa_uniform_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&current_input_texture.view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut jfa_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("JFA pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &current_output_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            jfa_pass.set_pipeline(&self.jfa_pipeline);
            jfa_pass.set_bind_group(0, &jfa_bindgroup, &[]);
            jfa_pass.draw(0..3, 0..1);
        }
        // Make it all into a distance field
        {
            swap(&mut current_input_texture, &mut current_output_texture);
            let df_bindgroup = device.create_bind_group(&BindGroupDescriptor {
                label: Some("distance field bindgroup"),
                layout: &self.distance_field_bg_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&current_input_texture.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            let mut df_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Distance field pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_texture_view,
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
            df_pass.set_pipeline(&self.distance_field_pipeline);
            df_pass.set_bind_group(0, &df_bindgroup, &[]);
            df_pass.draw(0..3, 0..1);
        }
    }
}
