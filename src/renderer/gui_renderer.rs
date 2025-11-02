use egui::FullOutput;
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use wgpu::{CommandEncoder, Device, Queue, RenderPassDescriptor, TextureFormat, TextureView};
use winit::window::Window;

pub struct GuiRenderer {
    renderer: egui_wgpu::Renderer,
    pub input_state: egui_winit::State,
    egui_output: Option<FullOutput>,
}

impl GuiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        let renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            RendererOptions {
                msaa_samples,
                depth_stencil_format: output_depth_format,
                ..Default::default()
            }
        );

        let gui_ctx = egui::Context::default();
        let id = gui_ctx.viewport_id();

        let input_state = egui_winit::State::new(gui_ctx, id, &window, None, None, None);
        Self {
            renderer,
            input_state,
            egui_output: None,
        }
    }
    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        target_view: &TextureView,
        window: &Window,
        screen_desc: ScreenDescriptor,
    ) {
        let egui_output = self.egui_output.take();
        if let Some(egui_output) = egui_output {
            self.input_state
                .handle_platform_output(window, egui_output.platform_output);
            let tris = self
                .input_state
                .egui_ctx()
                .tessellate(egui_output.shapes, egui_output.pixels_per_point);

            for (id, delta) in &egui_output.textures_delta.set {
                self.renderer.update_texture(device, queue, *id, delta);
            }
            self.renderer
                .update_buffers(device, queue, encoder, &tris, &screen_desc);
            {
                let gui_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    label: Some("egui render pass"),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                self.renderer.render(&mut gui_pass.forget_lifetime(), &tris, &screen_desc);
            }
        } else {
            log::warn!("Trying to render gui without a full output");
        }
    }
    pub fn prepare(&mut self, egui_output: FullOutput) {
        self.egui_output = Some(egui_output);
    }
}
