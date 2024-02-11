use crate::renderer::Renderer;
use std::sync::Arc;
use winit::{event::{KeyEvent, WindowEvent, DeviceEvent}, window::Window};

pub struct ApplicationState {
    pub renderer: Renderer,
}

impl ApplicationState {
    pub async fn new(window: Arc<Window>) -> Self {
        Self {
            renderer: Renderer::new(window).await,
        }
    }
    pub fn draw(&mut self) {
        // TODO: update renderer buffers and such
        self.renderer.render();
    }
    pub fn update(&mut self, dt_seconds: f64) {
        
    }

    // TODO: consider moving to input struct
    pub fn key_input(&mut self, event: &KeyEvent) {
        // TODO: handle keyboard input
    }
    pub fn mouse_input(&mut self, event: &WindowEvent) -> bool {
        // TODO: handle mouse input
        match event {
            WindowEvent::MouseInput { device_id, state, button } => {true},
            WindowEvent::MouseWheel { device_id, delta, phase } => {true},
            _ => {false}
        }
    }
    pub fn mouse_movement(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                // TODO: handle mouse motion
            },
            _ => {}
        }
    }
}
