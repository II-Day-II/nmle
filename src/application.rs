use crate::renderer::Renderer;
use log::{debug, trace};
use std::sync::Arc;
use winit::{
    event::{DeviceEvent, KeyEvent, WindowEvent},
    window::Window,
};

pub struct ApplicationState {
    pub renderer: Renderer,
}

impl ApplicationState {
    pub async fn new(window: Arc<Window>) -> Self {
        Self {
            renderer: Renderer::new(window).await,
        }
    }
    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        // TODO: update renderer buffers and such
        self.renderer.render()
    }
    pub fn _update(&mut self, dt_seconds: f64) {
        trace!("Update called with dt={}", dt_seconds);
    }

    // TODO: consider moving to input struct
    pub fn key_input(&mut self, event: &KeyEvent) {
        // TODO: handle keyboard input
        let KeyEvent {
            physical_key,
            state,
            ..
        } = event;
        debug!("Got key event {:?}", event);
        match physical_key {
            _ => {
                if state.is_pressed() {
                } else {
                }
            }
        }
    }
    pub fn mouse_input(&mut self, event: &WindowEvent) -> bool {
        // TODO: handle mouse input
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                debug!("Got mouse input {:?}, {:?}", button, state);
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                debug!("Got mousewheel delta {:?}", delta);
                true
            }
            _ => false,
        }
    }
    pub fn mouse_movement(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                // TODO: handle mouse motion
                trace!("Got mouse movement {}, {}", dx, dy);
            }
            _ => {}
        }
    }
}
