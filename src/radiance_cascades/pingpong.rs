use wgpu::{Device, Extent3d, Texture, TextureDescriptor, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use vek::Vec2;
pub struct PingPongTex {
    pub texture: Texture,
    pub view: TextureView,
}
impl PingPongTex {
    pub fn new(device: &Device, size: Vec2<u32>, label: &str, format_override: Option<TextureFormat>) -> Self {
        let format = if let Some(f) = format_override {f} else {TextureFormat::Bgra8Unorm};
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
            format,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[format.add_srgb_suffix(), format.remove_srgb_suffix()],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self { texture, view }
    }
}