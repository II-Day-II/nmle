use bytemuck::{Pod, Zeroable};
use vek::{FrustumPlanes, Mat4, Vec2};

use crate::application::input::Input;

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct PanAndZoom {
    pub position: Vec2<f32>,
    pub zoom: f32,
    aspect: f32,
}
pub struct Camera {
    pub position: Vec2<f32>,
    pub zoom: f32,
}

impl Camera {
    const SCROLL_SPEED: f32 = 0.05;

    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 0.2,
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
        proj * scale * translation
    }

    pub fn pan_and_zoom_data(&self, aspect: f32) -> PanAndZoom {
        PanAndZoom {
            position: self.position,
            zoom: self.zoom,
            aspect: aspect,
        }
    }
    pub fn pan(&mut self, input: &Input, window_size: Vec2<f32>) {
        if input.clicking[0] {
            let manual_delta = Vec2::new(input.current_mouse_pos.x, input.current_mouse_pos.y) - Vec2::new(input.last_mouse_pos.x, input.last_mouse_pos.y);
            let auto_delta = Vec2::<f64>::from(input.mouse_delta).as_();
            let normalized_delta = Vec2::new(1.0, -1.0) * (auto_delta + manual_delta.as_()) / window_size;
            self.position += normalized_delta * Vec2::new(window_size.x / window_size.y, 1.0) / self.zoom;
        }
    }
    pub fn zoom(&mut self, input: &Input) {
        self.zoom *= 1.0 + Self::SCROLL_SPEED * input.scroll_delta as f32;
        self.zoom = self.zoom.clamp(0.01, 10.0);
    }
}
