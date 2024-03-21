use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, Device, Queue, RenderPassDescriptor, TextureFormat, TextureView};
use winit::window::Window;

pub struct GuiRenderer {
    renderer: egui_wgpu::Renderer,
    pub input_state: egui_winit::State,
    pub ui_func: Box<dyn FnOnce(&egui::Context)>,
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
            output_depth_format,
            msaa_samples,
        );

        let gui_ctx = egui::Context::default();
        let id = gui_ctx.viewport_id();

        let input_state = egui_winit::State::new(gui_ctx, id, &window, None, None);
        Self {
            renderer,
            input_state,
            ui_func: Box::new(|_| {}),
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
        // ONLY use of the window. I should not have to store it.
        let raw_input = self.input_state.take_egui_input(&window);
        let egui_output = self.input_state.egui_ctx().run(raw_input, |ctx| {
            // TODO: how tf do I get state from ApplicationState visible in here??
            egui::Window::new("Streamline CFD")
                .default_open(true)
                .show(&ctx, |ui| {
                    ui.heading("Hello World");
                });
        });
        self.input_state
            .handle_platform_output(&window, egui_output.platform_output);
        let tris = self
            .input_state
            .egui_ctx()
            .tessellate(egui_output.shapes, egui_output.pixels_per_point);

        for (id, delta) in &egui_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, &delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_desc);
        {
            let mut gui_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer.render(&mut gui_pass, &tris, &screen_desc);
        }
    }
}
