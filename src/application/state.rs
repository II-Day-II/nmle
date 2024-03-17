use super::camera::Camera;
use crate::renderer::{Renderer, Vertex};
use log::{debug, trace};
use std::sync::Arc;
use vek::{Mat4, Vec4};
use wgpu::BufferUsages;
use winit::{
    event::{KeyEvent, WindowEvent},
    window::Window,
};

pub struct ApplicationState {
    camera: Camera,
    pub renderer: Renderer,
    theta: f32,
}

impl ApplicationState {
    pub async fn new(window: Arc<Window>) -> Self {
        let mut renderer = Renderer::new(window).await;
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
            renderer,
            camera,
            theta: 0.0,
        }
    }
    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        // TODO: update renderer buffers and such
        self.renderer.render()
    }
    pub fn update(&mut self, dt_seconds: f64) {
        trace!("Update called with dt={}", dt_seconds);
        self.theta += dt_seconds as f32 * std::f32::consts::PI;
        let view_proj = self.camera.get_matrix(self.renderer.aspect());
        let rot = Mat4::rotation_z(self.theta);
        let mat = view_proj * rot;
        self.renderer
            .write_buffer("camera", bytemuck::cast_slice(mat.as_col_slice()));
    }

    // TODO: consider moving to input struct
    pub fn key_input(&mut self, event: &KeyEvent) {
        // TODO: handle keyboard input
        let KeyEvent {
            physical_key,
            state,
            ..
        } = event;
        trace!("Got key event {:?}", event);
        match physical_key {
            _ => {
                if state.is_pressed() {
                } else {
                }
            }
        }
    }
    // handle a window event related to the mouse. Return true if the event was handled, false if it should be done in the event loop
    pub fn mouse_input(&mut self, event: &WindowEvent) -> bool {
        // TODO: handle mouse input
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                trace!("Got mouse input {:?}, {:?}", button, state);
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                trace!("Got mousewheel delta {:?}", delta);
                true
            }
            // Need to handle mouse movement here as well in order to keep track of where the mouse is.
            WindowEvent::CursorMoved { position, .. } => {
                trace!("Mouse moved to {:?}", position);
                true
            }
            _ => false,
        }
    }
    // handle raw physical movement of the mouse.
    pub fn mouse_movement(&mut self, delta: (f64, f64)) {
        // TODO: handle mouse motion
        let (dx, dy) = delta;
        trace!("Got mouse movement {}, {}", dx, dy);
    }
}
