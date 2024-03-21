use super::{camera::Camera, input::Input};
use crate::renderer::{Renderer, Vertex};
use log::{debug, trace};
use std::sync::Arc;
use vek::{Mat4, Vec4};
use wgpu::BufferUsages;
use winit::{event::Event, window::Window};

pub struct ApplicationState {
    camera: Camera,
    pub renderer: Renderer,
    input: Input,

    theta: f32,
}

impl ApplicationState {
    pub async fn new(window: Arc<Window>) -> Self {
        let input = Input::new();
        let mut renderer = Renderer::new(window.clone()).await;
        let vertices = [
            Vertex {
                position: Vec4::new(-0.5, -0.5, 0.0, 1.0),
                uv: Vec4::new(0.0, 0.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec4::new(0.5, -0.5, 0.0, 1.0),
                uv: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec4::new(-0.5, 0.5, 0.0, 1.0),
                uv: Vec4::new(0.0, 1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec4::new(0.5, 0.5, 0.0, 1.0),
                uv: Vec4::new(1.0, 1.0, 0.0, 0.0),
            },
        ];
        let indices = [0, 1, 2, 1, 3, 2];
        renderer.add_renderable("default_rect".into(), "Default".into(), &indices, &vertices);
        let camera = Camera::new();
        renderer.add_global_buffer(
            "camera".into(),
            bytemuck::cast_slice(camera.get_matrix(renderer.aspect()).as_col_slice()),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_pass("Default".into());

        debug!("Application state initialized");
        Self {
            input,
            renderer,
            camera,
            theta: 0.0,
        }
    }
    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update_gui();
        // TODO: update renderer buffers and such
        self.renderer.render()
    }

    fn update_gui(&mut self) {}

    pub fn update(&mut self, dt_seconds: f64) {
        trace!("Update called with dt={}", dt_seconds);
        self.theta += dt_seconds as f32 * std::f32::consts::PI;
        let view_proj = self.camera.get_matrix(self.renderer.aspect());
        let rot = Mat4::rotation_z(self.theta);
        let mat = view_proj * rot;
        self.renderer
            .write_buffer("camera", bytemuck::cast_slice(mat.as_col_slice()));
    }
    pub fn handle_event_input(&mut self, window: &Window, event: &Event<()>) -> bool {
        if let Event::WindowEvent { window_id, event } = event {
            if *window_id == window.id() {
                self.renderer
                    .gui_renderer
                    .input_state
                    .on_window_event(window, event);
            }
        }
        self.input.handle(&window.id(), event)
    }
}
