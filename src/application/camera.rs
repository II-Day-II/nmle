use bytemuck::{Pod, Zeroable};
use vek::{FrustumPlanes, Mat4, Vec2};

use crate::application::input::Input;

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct Camera {
    pub position: Vec2<f32>,
    pub zoom: f32,
}

impl Camera {
    const SCROLL_SPEED: f32 = 0.1;
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    pub fn get_matrix(&self, aspect: f32) -> Mat4<f32> {
        let proj: Mat4<f32> = Mat4::orthographic_without_depth_planes(FrustumPlanes {
            left: -1.0 * aspect,
            right: 1.0 * aspect,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 0.0,
        });
        let scale = Mat4::scaling_3d(self.zoom);
        let translation: Mat4<f32> = Mat4::translation_2d(self.position);
        proj * translation * scale
    }

    pub fn pan_and_zoom_data(&self) -> Self {
        Self {..*self}
    }
    pub fn pan(&mut self, input: &Input, window_size: Vec2<f32>) {
        if input.clicking[0] {
            let manual_delta = Vec2::new(input.current_mouse_pos.x, input.current_mouse_pos.y) - Vec2::new(input.last_mouse_pos.x, input.last_mouse_pos.y);
            let auto_delta = Vec2::<f64>::from(input.mouse_delta).as_();
            let normalized_delta = Vec2::new(1.0, -1.0) * (auto_delta + manual_delta.as_()) / window_size;
            self.position += normalized_delta;
        }
    }
    pub fn zoom(&mut self, input: &Input) {
        self.zoom += Self::SCROLL_SPEED * input.scroll_delta as f32;
        self.zoom = self.zoom.clamp(0.001, 10.0);
    }
}
