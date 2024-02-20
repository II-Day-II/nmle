use vek::{FrustumPlanes, Mat4, Vec2};

pub struct Camera {
    offset: Vec2<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset: Vec2::new(0.0, 0.0)
        }
    }

    pub fn get_matrix(&self) -> Mat4<f32> {
        let proj: Mat4<f32> = Mat4::orthographic_without_depth_planes(FrustumPlanes{
            left: -1.0, right: 1.0, bottom: -1.0, top: 1.0, near: 0.0, far: 0.0
        });
        let trans: Mat4<f32> = Mat4::translation_2d(self.offset);
        proj * trans
    }
}