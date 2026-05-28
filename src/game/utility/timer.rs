#[derive(Debug)]
pub struct Timer{
    time: f32,
    start: f32,
}

impl Timer {
    pub fn new(start: f32) -> Self {
        Self { time: start, start }
    }

    pub fn step(&mut self, delta_time: f32) -> bool {
        self.time -= delta_time;
        self.time <= 0.0
    }

    pub fn reset(&mut self) {
        self.time = self.start;
    }

    pub fn finished(&self) -> bool {
        self.time <= 0.0
    }
}