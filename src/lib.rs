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
        .filter_level(log::LevelFilter::Warn) // dependency log level
        .filter_module(module_path!(), log::LevelFilter::Debug) // current application log level
        .init();
    test_logging();

    // create window and eventloop
    let event_loop = EventLoop::new()?;
    let builder = WindowBuilder::new().with_title(TITLE);
    let window = builder.build(&event_loop)?;
    // TODO: wasm stuff
    // try wasm-server-runner https://github.com/jakobhellermann/wasm-server-runner

    let window = Arc::new(window);

    let mut state = application::ApplicationState::new(window.clone()).await;
    let mut start_time = instant::Instant::now();

    // TODO: determine if this is needed
    event_loop.set_control_flow(ControlFlow::Wait);
    // run the event loop
    event_loop.run(move |e_event, elwt| {
        // timing
        let now = instant::Instant::now();
        let delta = now - start_time;
        start_time = now;
        // deal with input
        if !state.handle_event_input(&window, &e_event) {
            // TODO: find out if you can skip all the id == id checks without nested match
            match e_event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    window_id,
                } if window_id == window.id() => match state.draw() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.renderer.resize(state.renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => error!("{}", e),
                },
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    window_id,
                } if window_id == window.id() => state.renderer.resize(new_size),
                Event::AboutToWait => {
                    // TODO: determine if redraw needed
                    let redraw_needed = true;
                    if redraw_needed {
                        window.request_redraw();
                        state.update(delta.as_secs_f64());
                    }
                }
                _ => {}
            }
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
