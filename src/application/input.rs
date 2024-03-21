use log::trace;
use winit::{
    event::{DeviceEvent, Event, KeyEvent, WindowEvent},
    window::WindowId,
};

pub struct Input {
    //TODO: Store relevant input state here
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(&mut self, w_id: &WindowId, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                    },
            } if window_id == w_id => self.keyboard_input(event),
            Event::WindowEvent { window_id, event } if window_id == w_id => self.mouse_input(event),
            Event::DeviceEvent {
                device_id: _,
                event: DeviceEvent::MouseMotion { delta },
            } => self.mouse_movement(delta),
            _ => false,
        }
    }
    pub fn keyboard_input(&mut self, event: &KeyEvent) -> bool {
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
        true
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
    pub fn mouse_movement(&mut self, delta: &(f64, f64)) -> bool {
        // TODO: handle mouse motion
        let (dx, dy) = delta;
        trace!("Got mouse movement {}, {}", dx, dy);
        true
    }
}
