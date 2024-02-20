use log::debug;
use wgpu::{include_spirv, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, ColorWrites, Device, RenderPass, ShaderStages, SurfaceConfiguration};


use super::{pipeline::{PipelineBuilder, Pipeline}, renderable::Renderable};

macro_rules! include_shader {
    ($file:expr) => {
        include_spirv!(concat!(env!("OUT_DIR"), $file))
    };
}
pub struct DefaultPass {
    pipeline: Pipeline,
    buffers: Vec<Buffer>,
    bind_groups: Vec<BindGroup>,
}

impl DefaultPass {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {

        let mut buffers = Vec::new();
        let mut bind_groups = Vec::new();

        // TODO: see if this works in web
        let v_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.vert.spv"));
        let f_shader_spv =
            device.create_shader_module(include_shader!("/shaders/default_glsl.frag.spv"));
        // let default_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor{
        //     label: Some("default bind group"),
        //     entries: &[BindGroupLayoutEntry {
        //         binding: 0,
        //         visibility: ShaderStages::VERTEX,
        //         ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        //         count: None,
        //     }]
        // });
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
            // .add_bind_group_layout(&default_bind_group_layout)
            .build(&device);

        debug!("Default pipeline constructed");
        Self {
            pipeline,
            buffers,
            bind_groups,
        }
    }
    pub fn draw<'a, 'b>(
            &'a self,
            render_pass: &mut RenderPass<'b>,
            renderables: impl Iterator<Item = &'a Renderable>
        ) -> Result<(), wgpu::SurfaceError> 
        where 'a : 'b {
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