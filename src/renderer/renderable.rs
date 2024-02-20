use bytemuck::{Pod, Zeroable};
use vek::Vec4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages, Device, RenderPass,
};

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct Vertex {
    pub position: Vec4<f32>,
    pub uv: Vec4<f32>,
}

impl Vertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub struct Renderable {
    pub pipeline_tag: String,
    vtx_buffer: wgpu::Buffer,
    idx_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Renderable {
    pub fn new(
        device: &Device,
        vertices: &[Vertex],
        indices: &[u16],
        pipeline_tag: String,
    ) -> Self {
        let num_indices = indices.len() as u32;
        let vtx_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("VertexBuffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let idx_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("IndexBuffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        Self {
            pipeline_tag,
            vtx_buffer,
            idx_buffer,
            num_indices,
        }
    }
    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vtx_buffer.slice(..));
        render_pass.set_index_buffer(self.idx_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
