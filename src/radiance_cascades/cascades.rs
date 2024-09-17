use vek::Vec2;
use wgpu::{
    include_spirv, util::{BufferInitDescriptor, DeviceExt}, 
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, 
    BufferBinding, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d, 
    FragmentState, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState, 
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, 
    Sampler, SamplerDescriptor, ShaderStages, TextureDescriptor, TextureFormat, TextureUsages, TextureView, VertexState
};

use crate::renderer::include_shader;

use super::pingpong::PingPongTex;
pub struct RadianceCascades {
    pub params: CascadeParams,

    // gi_pipeline: RenderPipeline,
    // gi_bg_layout: BindGroupLayout,
    
    sampler: Sampler,
    
    cascades_pipeline: RenderPipeline,
    cascades_bg_layouts: [BindGroupLayout; 2],
    cascade_textures: [PingPongTex; 2],
    // cascade_tex: Texture,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeParams {
    pub start_interval: f32,
    pub interval_size: f32,
    pub angle_offset: f32,
    pub cascade_count: u32, 
    pub base_ray_count: u32,
    pub ray_count: u32, // set automatically in cascade.run
}
impl CascadeParams {
    pub fn new() -> Self {
        Self {
            start_interval: 0.0,
            interval_size: 0.125,
            angle_offset: 0.5,
            cascade_count: 2,
            base_ray_count: 4,
            ray_count: 0
        }
    }
}

impl RadianceCascades {
    pub fn new(device: &Device, screen_size: Vec2<f32>) -> Self {

        let _cascade_tex = device.create_texture(&TextureDescriptor { 
            label: Some("cascadeTex"), 
            size: Extent3d {width: screen_size.x as u32, height: screen_size.y as u32, depth_or_array_layers: 1}, // TODO: see if use multiple layers instead?
            mip_level_count: 1, // TODO: see if use multiple mips instead?
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm, // TODO: decide
            view_formats: &[TextureFormat::Bgra8Unorm],
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, // TODO: add the rest
        });
        let cascade_tex_0 = PingPongTex::new(&device, Vec2::new(screen_size.x as u32, screen_size.y as u32), "cascade tex 0", Some(TextureFormat::Bgra8UnormSrgb));
        let cascade_tex_1 = PingPongTex::new(&device, Vec2::new(screen_size.x as u32, screen_size.y as u32), "cascade tex 1", Some(TextureFormat::Bgra8UnormSrgb));
        let cascade_textures = [cascade_tex_0, cascade_tex_1];

        let fs_tri_module = device.create_shader_module(include_shader!("/shaders/fullscreen_tri.vert.spv"));
        let cascades_module = device.create_shader_module(include_shader!("/shaders/cascade_gi.frag.spv"));
            
        // let gi_module = device.create_shader_module(include_shader!("/shaders/gi.frag.spv"));
        // let gi_bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        //     label: Some("gi bg layout"),
        //     entries: &[
        //         BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //                 multisampled: false,
        //             },
        //             count: None,
        //         },
        //         BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //                 multisampled: false,
        //             },
        //             count: None,
        //         },
        //         BindGroupLayoutEntry {
        //             binding: 2,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        // });
        // let gi_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        //     label: Some("gi_pipeline_layout"),
        //     bind_group_layouts: &[&gi_bg_layout],
        //     push_constant_ranges: &[],
        // });
        // let gi_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        //     label: Some("gi_pipeline"),
        //     layout: Some(&gi_pipeline_layout),
        //     vertex: VertexState {
        //         module: &fs_tri_module,
        //         entry_point: "main",
        //         buffers: &[],
        //     },
        //     primitive: PrimitiveState {
        //         topology: wgpu::PrimitiveTopology::TriangleList,
        //         strip_index_format: None,
        //         front_face: wgpu::FrontFace::Ccw,
        //         conservative: false,
        //         cull_mode: Some(wgpu::Face::Back),
        //         unclipped_depth: false,
        //         polygon_mode: wgpu::PolygonMode::Fill,
        //     },
        //     depth_stencil: None,
        //     multisample: MultisampleState {
        //         count: 1,
        //         mask: !0,
        //         alpha_to_coverage_enabled: false,
        //     },
        //     fragment: Some(FragmentState {
        //         module: &gi_module,
        //         entry_point: "main",
        //         targets: &[Some(ColorTargetState {
        //             format: TextureFormat::Bgra8UnormSrgb, // TODO: fix this,
        //             blend: None,
        //             write_mask: ColorWrites::ALL,
        //         })],
        //     }),
        //     multiview: None,
        // });
        let cascades_bg0_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("cascades bg0 layout"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let cascades_bg1_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("cascades bg1 layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None,
            }],
        });
        let cascades_bg_layouts = [cascades_bg0_layout, cascades_bg1_layout];
        let cascades_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("cascades pipeline layout"),
            bind_group_layouts: &[&cascades_bg_layouts[0], &cascades_bg_layouts[1]],
            push_constant_ranges: &[],
        });

        let cascades_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("cascades pipeline"),
            layout: Some(&cascades_pipeline_layout),
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
                module: &cascades_module,
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
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mut rc = Self {
            params: CascadeParams::new(),
            cascades_pipeline,
            cascades_bg_layouts,
            // gi_pipeline,
            // gi_bg_layout,

            cascade_textures,
            sampler,
        };
        rc.resize(&device, screen_size);
        rc
    }

    pub fn resize(&mut self, device: &Device, screen_size: Vec2<f32>) {
        let branching_factor = 2.0f32;
        let interval0 = 4.0; // TODO: what should this be?
        let diagonal = screen_size.distance(Vec2::new(0.0, 0.0)); // no length()?
        let factor = (diagonal / interval0).log(branching_factor).ceil();
        let start_interval = (interval0 * branching_factor.powf(factor)) / (branching_factor - 1.0);
        let cascade_count = start_interval.log(branching_factor).ceil() as u32;
        self.params.cascade_count = cascade_count;
        //self.params.start_interval = start_interval; // TODO: figure this shit out 
        self.cascade_textures[0] = PingPongTex::new(&device, Vec2::new(screen_size.x as u32, screen_size.y as u32), "cascade tex 0", Some(TextureFormat::Bgra8UnormSrgb));
        self.cascade_textures[1] = PingPongTex::new(&device, Vec2::new(screen_size.x as u32, screen_size.y as u32), "cascade tex 1", Some(TextureFormat::Bgra8UnormSrgb));
    }

    pub fn run(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        input_render_view: &TextureView,
        input_distance_view: &TextureView,
        output_tex_view: &TextureView,
    ) {
        let mut curr_cascade_idx = 0;
        for i in (1..=2).rev() {
            self.params.ray_count = if i < 1 {self.params.base_ray_count} else {self.params.base_ray_count.pow(i)};
            let cascades_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("cascades bg1 unform buffer"),
                contents: bytemuck::bytes_of(&self.params),
                usage: BufferUsages::UNIFORM,
            });
            let cascades_bg0 = device.create_bind_group(&BindGroupDescriptor {
                label: Some("cascades bg0"),
                layout: &self.cascades_bg_layouts[0],
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
                        resource: wgpu::BindingResource::TextureView(&self.cascade_textures[1 - curr_cascade_idx].view),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ]
            });
            let cascades_bg1 = device.create_bind_group(&BindGroupDescriptor {
                label: Some("cascadese bg1"),
                layout: &self.cascades_bg_layouts[1],
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(BufferBinding {
                            buffer: &cascades_buffer,
                            offset: 0,
                            size: None,
                        }),
                    }
                ]
            });
            let target_view = if i > 1 {&self.cascade_textures[curr_cascade_idx].view} else {output_tex_view};
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("cascade pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &target_view,
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
            pass.set_pipeline(&self.cascades_pipeline);
            pass.set_bind_group(0, &cascades_bg0, &[]);
            pass.set_bind_group(1, &cascades_bg1, &[]);
            pass.draw(0..3, 0..1);
            curr_cascade_idx = 1 - curr_cascade_idx;
        }
        // let gi_bindgroup = device.create_bind_group(&BindGroupDescriptor {
        //     label: Some("gi_bindgroup"),
        //     layout: &self.gi_bg_layout,
        //     entries: &[
        //         BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&input_render_view),
        //         },
        //         BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(&input_distance_view),
        //         },
        //         BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::Sampler(&self.sampler),
        //         },
        //     ],
        // });
        // let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        //     label: Some("GI pass"),
        //     color_attachments: &[Some(RenderPassColorAttachment {
        //         view: &output_tex_view,
        //         resolve_target: None,
        //         ops: Operations {
        //             load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
        //             store: wgpu::StoreOp::Store,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        //     timestamp_writes: None,
        //     occlusion_query_set: None,
        // });
        // pass.set_pipeline(&self.gi_pipeline);
        // pass.set_bind_group(0, &gi_bindgroup, &[]);
        // pass.draw(0..3, 0..1);
    }
}
