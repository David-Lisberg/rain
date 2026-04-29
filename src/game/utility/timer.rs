pub struct Timer(pub f32);

impl Timer {
    pub fn step(&mut self, delta_time: f32) -> bool {
        self.0 -= delta_time;
        self.0 <= 0.0
    }
}