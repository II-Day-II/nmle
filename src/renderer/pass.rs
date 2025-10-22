use std::sync::Arc;

use log::debug;
use wgpu::{
    include_spirv, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages,
    ColorWrites, Device, RenderPass, ShaderStages, SurfaceConfiguration,
};

use super::{
    pipeline::{Pipeline, PipelineBuilder},
    renderable::Renderable,
};

macro_rules! include_shader {
    ($file:expr) => {
        include_spirv!(concat!(env!("OUT_DIR"), $file))
    };
}
pub struct DefaultPass {
    pipeline: Pipeline,
    bind_groups: Vec<BindGroup>,
}


fn make_entries(buffers: &Vec<Buffer>) -> (Vec<BindGroupLayoutEntry>, Vec<BindGroupEntry<'_>>) {
    let layout_entries = buffers
        .iter()
        .enumerate()
        .map(|(i, buf)| {
            BindGroupLayoutEntry {
                binding: i as u32,
                visibility: ShaderStages::VERTEX_FRAGMENT, // TODO: reduce,
                ty: BindingType::Buffer {
                    ty: if buf.usage().contains(BufferUsages::UNIFORM) {
                        BufferBindingType::Uniform
                    } else {
                        unimplemented!("deal with storage buffers in bindgroupLayoutEntry")
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        })
        .collect::<Vec<_>>();
    let bindgroup_entries = buffers
        .iter()
        .enumerate()
        .map(|(i, buf)| BindGroupEntry {
            binding: i as u32,
            resource: buf.as_entire_binding(),
        })
        .collect::<Vec<_>>();
    (layout_entries, bindgroup_entries)
}

pub fn draw_grid<'a, 'b>(render_pass: &mut RenderPass<'b>, grid: &'a DefaultPass) 
where 'a: 'b {
    render_pass.set_pipeline(&grid.pipeline.pipeline);
    for (i, group) in grid.bind_groups.iter().enumerate() {
        render_pass.set_bind_group(i as u32, group, &[]);
    }
    render_pass.draw(0..3,0..1);
}

impl DefaultPass {

    pub fn new_grid(
        device: &Device,
        config: &SurfaceConfiguration,
        buffers: &Vec<Buffer>,
    ) -> Self {
        let mut bind_groups = Vec::new();
        let (layout_entries, bindgroup_entries) = make_entries(buffers);
        
        let vertex_shader_spv = device.create_shader_module(include_shader!("/shaders/fullscreen_tri.vert.spv"));
        let vtx_entry_point = Some("main");
        let fragment_shader_spv = device.create_shader_module(include_shader!("/shaders/grid.frag.spv"));
        let bindgroup_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor { label: Some("Grid bindgroup layout"), entries: &layout_entries }
        );
        let pipeline = PipelineBuilder::new(&vertex_shader_spv, vtx_entry_point, false)
            .with_fragment_shader(&fragment_shader_spv, Some("main"))
            .with_cull_mode(wgpu::Face::Back)
            .add_fragment_target(Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE), // TODO: choose correct blend mode (may need to impl depth stencil stuff too)
                write_mask: ColorWrites::ALL,
            }))
            .add_bind_group_layout(&bindgroup_layout)
            .build(device);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("grid bind group"),
            layout: &bindgroup_layout,
            entries: &bindgroup_entries,
        });
        bind_groups.push(bind_group);
        Self {
            pipeline,
            bind_groups,
        }
    }

    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        buffers: &Vec<Buffer>,
    ) -> Self {
        let mut bind_groups = Vec::new();
        let (layout_entries, bindgroup_entries) = make_entries(buffers);
        // TODO: see if this works in web
        let v_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.vert.spv"));
        let f_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.frag.spv"));
        let default_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("default bind group"),
                entries: &layout_entries,
                // &[BindGroupLayoutEntry {
                //     binding: 0,
                //     visibility: ShaderStages::VERTEX,
                //     ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                //     count: None,
                // }]
            });
        // TODO: implement bind groups properly
        let mut builder = PipelineBuilder::new(&v_shader_spv, Some("main"), true);
        let pipeline = builder
            .with_fragment_shader(&f_shader_spv, Some("main"))
            .with_cull_mode(wgpu::Face::Back)
            .add_fragment_target(Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            }))
            .add_bind_group_layout(&default_bind_group_layout)
            .build(device);

        let default_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Default BindGroup"),
            layout: &default_bind_group_layout,
            entries: &bindgroup_entries,
            // &[BindGroupEntry{
            //     binding: 0,
            //     resource: BindingResource::Buffer(BufferBinding{
            //         buffer: &buffers[0],
            //         offset: 0,
            //         size: None,
            //     })
            // }]
        });
        bind_groups.push(default_bind_group);
        debug!("Default pipeline constructed");
        Self {
            pipeline,
            bind_groups,
        }
    }

    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &mut RenderPass<'b>,
        renderables: impl Iterator<Item = &'a Arc<Renderable>>,
    ) -> Result<(), wgpu::SurfaceError>
    where
        'a: 'b,
    {
        // 1. set the pipeline
        render_pass.set_pipeline(&self.pipeline.pipeline);
        // 2. set the bind groups
        for (i, group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, group, &[]);
        }
        // 3. draw the renderables
        for renderable in renderables {
            renderable.draw(render_pass);
        }

        Ok(())
    }
}
