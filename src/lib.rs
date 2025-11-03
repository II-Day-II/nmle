mod application;
mod renderer;

#[cfg(target_arch = "wasm32")]
use {
    winit::platform::web::WindowExtWebSys, wasm_bindgen::prelude::*
};

use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use winit::{
    event::*,
    event_loop::{EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
    window::{Window, WindowId},
};

struct AppWrap {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<application::ApplicationState>>,
    state: Option<application::ApplicationState>,
}

impl AppWrap {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<application::ApplicationState>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<application::ApplicationState> for AppWrap {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();
        window_attributes = window_attributes.with_title(TITLE);
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
        
            window_attributes = window_attributes.with_append(true);
        } 
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Window creation is allowed and possible"));

        #[cfg(not(target_arch = "wasm32"))]
        {
             // not on wasm, use pollster to get application state
             self.state = Some(pollster::block_on(application::ApplicationState::new(window)));
        }
        #[cfg(target_arch = "wasm32")]
        {
            let canvas = window.canvas().unwrap_throw();
            let style = &canvas.style();
            style.set_property("margin", "0px").unwrap_throw();
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy.send_event(application::ApplicationState::new(window.clone()).await).is_ok());
                });
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: application::ApplicationState) {
        event.renderer.resize(event.renderer.window.inner_size());
        event.renderer.window.request_redraw();
        self.state = Some(event);
    }

    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: DeviceId,
            event: DeviceEvent,
        ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };
        if let DeviceEvent::MouseMotion {delta} = event {
            state.input.mouse_movement(&delta);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.update();
        }    
    }
    
    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };
        let window = &state.renderer.window;
        let _ = state.renderer.gui_renderer.input_state.on_window_event(window, &event);
        state.input.mouse_input(&event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.renderer.resize(size),
            WindowEvent::RedrawRequested => {
                match state.draw() {
                    Err(wgpu::SurfaceError::Lost) => state.renderer.resize(state.renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => error!("{}", e),
                    Ok(_) => {},
                }
            }
            WindowEvent::KeyboardInput {event, .. } => {
                state.input.keyboard_input(&event);
            }
            _ => {},
        }
    }
}

const TITLE: &str = "nmle";

pub fn run() -> anyhow::Result<()> {
    // init logging
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Warn) // dependency log level
            .filter_module(module_path!(), log::LevelFilter::Debug) // current application log level
            .init();

    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).unwrap_throw();
    }
    test_logging();

    // create window and eventloop
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = AppWrap::new(
        #[cfg(target_arch = "wasm32")] &event_loop
    );
    event_loop.run_app(&mut app)?;
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

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*; 
    #[wasm_bindgen(start)]
    pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
        console_error_panic_hook::set_once();
        super::run().unwrap_throw();

        Ok(())
    }
}