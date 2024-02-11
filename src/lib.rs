mod application;
mod renderer;

use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const TITLE: &str = "nmle - New Matrix Learning Engine";

pub async fn run() -> anyhow::Result<()> {
    // init logging
    env_logger::Builder::new()
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::max())
        .init();
    test_logging();

    // create window and eventloop
    let event_loop = EventLoop::new()?;
    let builder = WindowBuilder::new().with_title(TITLE);
    let window = builder.build(&event_loop)?;
    // TODO: wasm stuff

    let window = Arc::new(window);

    let mut state = application::ApplicationState::new(window.clone()).await;

    // TODO: determine if this is needed
    event_loop.set_control_flow(ControlFlow::Wait);

    // run the event loop
    event_loop.run(move |e_event, elwt| {
        match e_event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            },
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                state.draw();
            },
            Event::AboutToWait => {
                // TODO: determine if redraw needed
                let redraw_needed = true;
                if redraw_needed {
                    window.request_redraw();
                }
            },
            Event::WindowEvent { window_id, event } if window_id == window.id() => if !state.mouse_input(&event) { 
                match event {
                    WindowEvent::Resized(new_size) => state.renderer.resize(new_size),
                    WindowEvent::KeyboardInput { event, .. } => {
                        // TODO: determine if the keyevent is enough or if we need 
                        // the device_id and is_synthetic as well
                        state.key_input(&event);
                    }
                    _ => {}
                }
            },
            Event::DeviceEvent {event, ..} => {
                state.mouse_movement(&event);
            }
            _ => {}
        }
    })?;
    Ok(())
}

fn test_logging() {
    let message = "hello logging";
    trace!("{}", message);
    debug!("{}", message);
    info!("{}", message);
    warn!("{}", message);
    error!("{}", message);
}
