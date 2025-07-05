use vek::{FrustumPlanes, Mat4, Vec2};

pub struct Camera {
    position: Vec2<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
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
        let trans: Mat4<f32> = Mat4::translation_2d(self.position);
        proj * trans
    }
}
