use vek::Vec2;
pub struct JumpFlood {
    num_passes: u32
}

impl JumpFlood {
    pub fn new(screen_size: Vec2<f32>) -> Self {
        let num_passes = screen_size.reduce_partial_max().log2().ceil() as u32;
        Self {
            num_passes,
        }
    }
    pub fn resize(&mut self, screen_size: Vec2<f32>) {
        self.num_passes = screen_size.reduce_partial_max().log2().ceil() as u32;
    }
}