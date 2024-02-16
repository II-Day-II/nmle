use crate::renderer::Renderer;
use log::{debug, trace};
use std::sync::Arc;
use winit::{
    event::{KeyEvent, WindowEvent},
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
    // handle a window event related to the mouse. Return true if the event was handled, false if it should be done in the event loop
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
