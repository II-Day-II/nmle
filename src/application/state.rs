use super::{camera::Camera, input::Input};
use crate::renderer::{Renderable, Renderer, Vertex};
use log::{debug, trace};
use std::sync::Arc;
use vek::{Mat4, Vec4};
use wgpu::BufferUsages;
use winit::{event::Event, window::Window};

pub struct ApplicationState {
    camera: Camera,
    pub renderer: Renderer,
    input: Input,

    matrix_stack: Vec<Mat4<f32>>,
    model: Model,
    theta: f32,
}

struct Model {
    _renderable: Arc<Renderable>,
    pub transform: Mat4<f32>,
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
        let _renderable = renderer.add_renderable("default_rect".into(), "Default".into(), &indices, &vertices);
        let model = Model {
            _renderable, 
            transform: Mat4::identity()
        };
        let matrix_stack = vec![Mat4::identity()];
        let camera = Camera::new();
        renderer.add_global_buffer(
            "camera".into(),
            bytemuck::cast_slice(camera.get_matrix(renderer.aspect()).as_col_slice()),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_global_buffer(
            "transform".into(),
            bytemuck::cast_slice(model.transform.as_col_slice()),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_pass("Default".into());

        debug!("Application state initialized");
        Self {
            input,
            renderer,
            camera,

            matrix_stack,
            model,
            theta: 0.0,
        }
    }
    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update_gui();
        // TODO: update renderer buffers and such
        self.renderer.render()
    }

    fn update_gui(&mut self) {
        let raw_input = self
            .renderer
            .gui_renderer
            .input_state
            .take_egui_input(&self.renderer.window);
        let egui_output = self
            .renderer
            .gui_renderer
            .input_state
            .egui_ctx()
            .run(raw_input, |ctx| {
                egui::Window::new("egui window").show(ctx, |ui| {
                    ui.heading("Transform stack");
                    let mut remove_index = None;
                    for (id, mat) in self.matrix_stack.iter_mut().enumerate() {
                        ui.group(|ui| {
                            let values = mat.as_mut_col_slice();
                            egui::Grid::new(&format!("Matrix_{id}")).show(ui, |ui| {
                                for y in 0..4usize {
                                    for x in 0..4usize {
                                        ui.add(egui::DragValue::new(&mut values[x*4 + y]).speed(0.01));
                                    }
                                    ui.end_row();
                                }
                            });
                            if id > 0 {
                                if ui.button("Remove matrix").clicked() {
                                    remove_index = Some(id);
                                }
                            }
                        });
                    }
                    if let Some(id) = remove_index {
                        self.matrix_stack.remove(id);
                    }
                    if ui.button("Add matrix").clicked() {
                        self.matrix_stack.push(Mat4::identity());
                    }
                    ui.separator();
                    ui.drag_angle(&mut self.theta);
                });
            });
        self.renderer.gui_renderer.prepare(egui_output);
    }


    pub fn update(&mut self, dt_seconds: f64) {
        trace!("Update called with dt={}", dt_seconds);
        // self.theta += dt_seconds as f32 * std::f32::consts::PI;
        let view_proj = self.camera.get_matrix(self.renderer.aspect());
        let rot = Mat4::rotation_z(self.theta);
        let mat = view_proj * rot;

        self.model.transform = self.matrix_stack.iter().fold(Mat4::identity(), |acc, &m| {acc * m}); // TODO: ordering of operations here is prob wrong

        self.renderer.write_buffer("transform", bytemuck::cast_slice(self.model.transform.as_col_slice()));

        self.renderer
            .write_buffer("camera", bytemuck::cast_slice(mat.as_col_slice()));
    }
    pub fn handle_event_input(&mut self, window: &Window, event: &Event<()>) -> bool {
        if let Event::WindowEvent { window_id, event } = event {
            if *window_id == window.id() {
                let _ = self
                    .renderer
                    .gui_renderer
                    .input_state
                    .on_window_event(window, event);
            }
        }
        self.input.handle(&window.id(), event)
    }
}
