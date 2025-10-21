use super::{camera::Camera, input::Input};
use crate::renderer::{Renderable, Renderer, Vertex};
use log::{debug, trace};
use std::sync::Arc;
use vek::{Mat4, Vec4, Vec3, Vec2};
use wgpu::BufferUsages;
use winit::{event::Event, window::Window};

#[derive(PartialEq)]
enum MatrixInteractionType {
    CustomMatrix, // just a mat4
    RotationMatrixZ(f32), // an angle to rotate about Z-axis
    TranslationMatrix2D(Vec2<f32>), // a vec2 to translate along XY plane
    ScaleMatrix2D(Vec2<f32>), // a vec2 to scale in XY
}

pub struct ApplicationState {
    camera: Camera,
    pub renderer: Renderer,
    input: Input,

    matrix_stack: Vec<(Mat4<f32>, MatrixInteractionType)>,
    show_full_matrix: bool,
    model: Model,
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
            Vertex::new(
                Vec4::new(-0.5, -0.5, 0.0, 1.0),
                Vec4::new(0.0, 0.0, 0.0, 0.0),
            ),
            Vertex::new(
                Vec4::new(0.5, -0.5, 0.0, 1.0),
                Vec4::new(1.0, 0.0, 0.0, 0.0),
            ),
            Vertex::new(
                Vec4::new(-0.5, 0.5, 0.0, 1.0),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
            ),
            Vertex::new(
                Vec4::new(0.5, 0.5, 0.0, 1.0),
                Vec4::new(1.0, 1.0, 0.0, 0.0),
            ),
        ];
        let indices = [0, 1, 2, 1, 3, 2];
        let _renderable = renderer.add_renderable("default_rect".into(), "Default".into(), &indices, &vertices);
        let model = Model {
            _renderable, 
            transform: Mat4::identity()
        };
        let matrix_stack = vec![(Mat4::identity(),MatrixInteractionType::CustomMatrix)];
        let camera = Camera::new();
        renderer.add_global_buffer(
            "camera".into(),
            0,
            bytemuck::cast_slice(camera.get_matrix(renderer.aspect()).as_col_slice()),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_global_buffer(
            "transform".into(),
            1,
            bytemuck::cast_slice(model.transform.as_col_slice()),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_global_buffer(
            "camera_pan_zoom".into(),
            2,
            bytemuck::cast_slice(&[camera.pan_and_zoom_data()]),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        renderer.add_pass("Default".into());

        debug!("Application state initialized");
        Self {
            input,
            renderer,
            camera,

            matrix_stack,
            show_full_matrix: false,
            model,
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
                egui::Window::new( "Debug").show(ctx, |ui| {
                    ui.label(format!("Camera position: {}", self.camera.position));
                    ui.label(format!("Camera zoom: {}", self.camera.zoom));
                    ui.label(format!("Mouse position: {:?}", self.input.current_mouse_pos));
                    ui.label(format!("Last position: {:?}", self.input.last_mouse_pos));
                    ui.label(format!("Mouse delta: {:?}", self.input.mouse_delta));
                });
                egui::Window::new("Controls")
                .max_height(self.renderer.size.height as f32 - 128.0)
                .default_width(128.0)
                .show(ctx, |ui| {
                    ui.heading("Transform stack");
                    
                    let mut remove_index = None;
                    
                    let mut from : Option<usize> = None;
                    let mut to : Option<usize> = None;
                    
                    let row_height = 5.5 + (self.show_full_matrix as u8 as f32) * ui.text_style_height(&egui::TextStyle::Body);
                    let total_rows = self.matrix_stack.len();
                    egui::ScrollArea::vertical().show_rows(ui, row_height, total_rows,|ui, row_range| {
                        for (idx, (mat, interaction_type)) in self.matrix_stack[row_range].iter_mut().enumerate() {
                            let frame = egui::Frame::default().inner_margin(1.0);
                            // specify zone for drag n drop
                            let (_, _dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
                                let item_id = egui::Id::new(("matrix_stack_drag_and_drop", idx));
                                // track dragging events
                                let response = ui.group(|ui| {
                                    // make list elements draggable by their names
                                    ui.dnd_drag_source(item_id, idx, |ui| {
                                        ui.label(
                                            match interaction_type {
                                                MatrixInteractionType::CustomMatrix => "Custom matrix",
                                                MatrixInteractionType::RotationMatrixZ(_) => "Rotation",
                                                MatrixInteractionType::ScaleMatrix2D(_) => "Scale",
                                                MatrixInteractionType::TranslationMatrix2D(_) => "Translation",
                                        });
                                    });

                                    // matrix list entry
                                    ui.group(|ui| {
                                        let values = mat.as_mut_col_slice();
                                        // add each element of the matrix as a dragvalue
                                        if *interaction_type == MatrixInteractionType::CustomMatrix {
                                            egui::Grid::new(&format!("Matrix_{idx}")).show(ui, |ui| {
                                                for y in 0..4usize {
                                                    if !self.show_full_matrix && y == 2 {
                                                        continue;
                                                    }
                                                    for x in 0..4usize {
                                                        if !self.show_full_matrix && x == 2 {
                                                            continue;
                                                        }
                                                        ui.add(egui::DragValue::new(&mut values[x*4 + y]).speed(0.01));
                                                    }
                                                    ui.end_row();
                                                }
                                            });
                                        }
                                        // add interaction for preset matrix
                                        match interaction_type {
                                            MatrixInteractionType::RotationMatrixZ(angle) => {
                                                ui.label("Angle:");
                                                ui.drag_angle(angle);
                                                *mat = Mat4::rotation_z(*angle);
                                            },
                                            MatrixInteractionType::TranslationMatrix2D(offset) => {
                                                ui.add(egui::DragValue::new(&mut offset.x).speed(0.01).prefix("x: "));
                                                ui.add(egui::DragValue::new(&mut offset.y).speed(0.01).prefix("y: "));
                                                *mat = Mat4::translation_2d(*offset);
                                            },
                                            MatrixInteractionType::ScaleMatrix2D(scale) => {
                                                ui.add(egui::DragValue::new(&mut scale.x).speed(0.01).prefix("x: "));
                                                ui.add(egui::DragValue::new(&mut scale.y).speed(0.01).prefix("y: "));
                                                *mat = Mat4::scaling_3d(Vec3::new(scale.x, scale.y, 1.0));
                                            },
                                            _ => {},
                                        }
                                        ui.allocate_space(ui.available_size());
                                    });
                                }).response;
                                // tag matrix for deletion
                                if idx > 0 {
                                    if ui.button("Remove matrix").clicked() {
                                        remove_index = Some(idx);
                                    }
                                }
                                // detect dragndrop
                                if let Some(_) = response.dnd_hover_payload::<usize>() {
                                    // handle dropped item
                                    if let Some(dragged_payload) = response.dnd_release_payload() {
                                        // item was dropped here
                                        from = Some(*dragged_payload);
                                        to = Some(idx);
                                    }
                                }
                            });
                        } // end for
                    });
                    
                    // rearrange matrices
                    if let (Some(from), Some(mut to)) = (from, to) {
                        let item = self.matrix_stack.remove(from);
                        to = to.min(self.matrix_stack.len());
                        self.matrix_stack.insert(to, item);
                    }

                    // remove matrix tagged for removal
                    if let Some(idx) = remove_index {
                        self.matrix_stack.remove(idx);
                    }

                    // final menu items
                    ui.menu_button("Add matrix", |ui| {
                        let custom_clicked = ui.button("Custom matrix").clicked();
                        let rotation_clicked = ui.button("Rotation").clicked();
                        let scale_clicked = ui.button("Scale").clicked();
                        let translation_clicked = ui.button("Translation").clicked();
                        let any_clicked = custom_clicked || rotation_clicked || scale_clicked || translation_clicked;
                        if custom_clicked {
                            self.matrix_stack.push((Mat4::identity(), MatrixInteractionType::CustomMatrix));
                        }
                        else if rotation_clicked {
                            self.matrix_stack.push((Mat4::identity(), MatrixInteractionType::RotationMatrixZ(0.0)));
                        }
                        else if scale_clicked {
                            self.matrix_stack.push((Mat4::identity(), MatrixInteractionType::ScaleMatrix2D(Vec2::new(1.0,1.0))));
                        }
                        else if translation_clicked {
                            self.matrix_stack.push((Mat4::identity(), MatrixInteractionType::TranslationMatrix2D(Vec2::new(0.0,0.0))));
                        }
                        if any_clicked {
                            ui.close_menu();
                        }
                    });
                    ui.checkbox(&mut self.show_full_matrix, "4x4 Matrices");
                    ui.label("Matrices are applied from bottom to top.");
                    ui.separator();
                });
            });
        self.renderer.gui_renderer.prepare(egui_output);
    }


    pub fn update(&mut self, dt_seconds: f64) {
        trace!("Update called with dt={}", dt_seconds);

        if !self.renderer.gui_renderer.input_state.egui_ctx().is_using_pointer() {
            self.camera.pan(&self.input, Vec2::new(self.renderer.size.width, self.renderer.size.height).as_());
            if !self.renderer.gui_renderer.input_state.egui_ctx().is_pointer_over_area() {
                self.camera.zoom(&self.input);
            }
        }

        self.input.update();

        let view_proj = self.camera.get_matrix(self.renderer.aspect());
        let mat = view_proj;

        self.model.transform = self.matrix_stack.iter().fold(Mat4::identity(), |acc, m| {acc * m.0}); 

        self.renderer.write_buffer("camera", bytemuck::cast_slice(mat.as_col_slice()));
        
        self.renderer.write_buffer("transform", bytemuck::cast_slice(self.model.transform.as_col_slice()));

        self.renderer.write_buffer("camera_pan_zoom", bytemuck::cast_slice(&[self.camera.pan_and_zoom_data()]));
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
