use log::trace;
use winit::{
    dpi::PhysicalPosition, event::{DeviceEvent, Event, KeyEvent, WindowEvent}, window::WindowId
};

pub struct Input {
    //TODO: Store relevant input state here
    pub current_mouse_pos: PhysicalPosition<f64>,
    pub last_mouse_pos: PhysicalPosition<f64>,
    pub clicking: [bool;3], // left, right, middle
    pub mouse_delta: (f64, f64),
    pub scroll_delta: f64,
}

impl Input {
    pub fn new() -> Self {
        Self {
            current_mouse_pos: PhysicalPosition::default(),
            last_mouse_pos: PhysicalPosition::default(),
            clicking: [false;3],
            mouse_delta: (0.0, 0.0),
            scroll_delta: 0.0
        }
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
                let idx = match button {
                    winit::event::MouseButton::Left => 0,
                    winit::event::MouseButton::Right => 1,
                    winit::event::MouseButton::Middle => 2,
                    _ => unimplemented!(),
                };
                let val = *state == winit::event::ElementState::Pressed;
                self.clicking[idx] = val;
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                trace!("Got mousewheel delta {:?}", delta);
                self.scroll_delta += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                        *y as f64
                    },
                    winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition{y, ..}) => {
                        y * 8.0
                    }
                };
                true
            }
            // Need to handle mouse movement here as well in order to keep track of where the mouse is.
            WindowEvent::CursorMoved { position, .. } => {
                trace!("Mouse moved to {:?}", position);
                self.current_mouse_pos = *position;
                true
            }
            _ => false,
        }
    }
    // handle raw physical movement of the mouse.
    pub fn mouse_movement(&mut self, delta: &(f64, f64)) -> bool {
        // TODO: handle mouse motion
        let (dx, dy) = delta;
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
        trace!("Got mouse movement {}, {}", dx, dy);
        true
    }

    pub fn update(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
        self.last_mouse_pos = self.current_mouse_pos;
    }
}
