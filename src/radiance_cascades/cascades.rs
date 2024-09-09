use vek::Vec2;
pub struct RadianceCascades {
    start_interval: f32,
    cascade_count: u8,
}

impl RadianceCascades {
    pub fn new(screen_size: Vec2<f32>) -> Self {
        let mut rc = Self {
            start_interval: 0.0,
            cascade_count: 0,
        };
        rc.resize(screen_size);
        rc
    }

    pub fn resize(&mut self, screen_size: Vec2<f32>) {
        let branching_factor = 2.0f32;
        let interval0 = 4.0; // TODO: what should this be?
        let diagonal = screen_size.distance(Vec2::new(0.0, 0.0)); // no length()?
        let factor = (diagonal / interval0).log(branching_factor).ceil();
        let start_interval = (interval0 * branching_factor.powf(factor)) / (branching_factor - 1.0);
        let cascade_count = start_interval.log(branching_factor).ceil() as u8;
        self.cascade_count = cascade_count;
        self.start_interval = start_interval;
    }
}
