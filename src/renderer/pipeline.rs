use wgpu::{
    BindGroupLayout, ColorTargetState, DepthStencilState, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModule, VertexBufferLayout, VertexState
};

use super::renderable::Vertex;

pub struct PipelineBuilder<'a> {
    pipeline_layout_descriptor: PipelineLayoutDescriptor<'a>,
    render_pipeline_descriptor: RenderPipelineDescriptor<'a>,
    fragment_targets: Vec<Option<ColorTargetState>>,
    bind_group_layouts: Vec<&'a BindGroupLayout>,
}

#[allow(dead_code)]
impl<'a> PipelineBuilder<'a> {
    pub fn new(vertex_shader: &'a ShaderModule, vtx_entry_point: Option<&'a str>) -> Self {
        const VTX_LAYOUT: VertexBufferLayout = Vertex::desc();
        let pipeline_layout_descriptor = PipelineLayoutDescriptor {
            label: Some("PipelineLayout"),
            bind_group_layouts: &[],   // TODO: get bind group layouts in here
            push_constant_ranges: &[], // are these a thing on not vulkan?
        };
        let render_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: None,
            vertex: VertexState {
                module: vertex_shader,
                entry_point: vtx_entry_point.unwrap_or("main"),
                buffers: &[VTX_LAYOUT],
            },
            primitive: PrimitiveState {
                // TODO: allow modification of more of this
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: None,
            fragment: None,
            multiview: None,
        };
        Self {
            pipeline_layout_descriptor,
            render_pipeline_descriptor,
            fragment_targets: vec![],
            bind_group_layouts: vec![],
        }
    }

    pub fn with_fragment_shader(
        &mut self,
        shader_module: &'a ShaderModule,
        entry_point: Option<&'a str>,
    ) -> &mut Self {
        self.render_pipeline_descriptor.fragment = Some(FragmentState {
            module: shader_module,
            entry_point: entry_point.unwrap_or("main"),
            targets: &[],
        });
        self
    }
    pub fn with_fragment_targets(&mut self, targets: &[Option<ColorTargetState>]) -> &mut Self {
        self.fragment_targets.extend_from_slice(targets);
        self
    }
    pub fn add_fragment_target(&mut self, target: Option<ColorTargetState>) -> &mut Self {
        self.fragment_targets.push(target);
        self
    }
    pub fn with_bind_group_layouts(&mut self, layouts: &[&'a BindGroupLayout]) -> &mut Self {
        self.bind_group_layouts.extend_from_slice(layouts);
        self
    }
    pub fn add_bind_group_layout(&mut self, layout: &'a BindGroupLayout) -> &mut Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn with_cull_mode(&mut self, cull_mode: Face) -> &mut Self {
        self.render_pipeline_descriptor.primitive.cull_mode = Some(cull_mode);
        self
    }

    pub fn with_depth_stencil(&mut self, depth_stencil: DepthStencilState) -> &mut Self {
        self.render_pipeline_descriptor.depth_stencil = Some(depth_stencil);
        self
    }
    pub fn build(&mut self, device: &Device) -> Pipeline {
        // TODO: try to get out of cloning ig 
        let mut layout_descriptor = self.pipeline_layout_descriptor.clone(); // probably suboptimal
        layout_descriptor.bind_group_layouts = &self.bind_group_layouts;
        let layout = device.create_pipeline_layout(&layout_descriptor);

        let mut render_pipeline_descriptor = self.render_pipeline_descriptor.clone(); // this is probably suboptimal
        render_pipeline_descriptor.layout = Some(&layout);
        if let Some(ref mut frag) = render_pipeline_descriptor.fragment {
            frag.targets = &self.fragment_targets;
        }

        let pipeline = device.create_render_pipeline(&render_pipeline_descriptor);
        Pipeline { layout, pipeline }
    }
}

pub struct Pipeline {
    pub pipeline: RenderPipeline,
    pub layout: PipelineLayout,
}
