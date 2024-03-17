use std::collections::HashMap;

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

impl DefaultPass {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        buffers: &HashMap<String, Buffer>,
    ) -> Self {
        let mut bind_groups = Vec::new();
        let layout_entries = buffers
            .values()
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
            .values()
            .enumerate()
            .map(|(i, buf)| BindGroupEntry {
                binding: i as u32,
                resource: buf.as_entire_binding(),
            })
            .collect::<Vec<_>>();

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
        let mut builder = PipelineBuilder::new(&v_shader_spv, Some("main"));
        let pipeline = builder
            .with_fragment_shader(&f_shader_spv, Some("main"))
            .with_cull_mode(wgpu::Face::Back)
            .add_fragment_target(Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            }))
            .add_bind_group_layout(&default_bind_group_layout)
            .build(&device);

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
        renderables: impl Iterator<Item = &'a Renderable>,
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
