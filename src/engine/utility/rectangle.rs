use glam::Vec2;

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y, 
            w, 
            h, 
        }
    }

    pub fn check_collision_point_rec(&self, point: impl Into<Vec2>) -> bool {
        let p = point.into();
        p.x < self.x + self.w &&
        p.x > self.x &&
        p.y < self.y + self.h &&
        p.y > self.y
    }
}

impl From<(f32, f32, f32, f32)> for Rect {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Rect { x: value.0, y: value.1, w: value.2, h: value.3 }
    }
}

impl From<[f32; 4]> for Rect {
    fn from(value: [f32; 4]) -> Self {
        Rect { x: value[0], y: value[1], w: value[2], h: value[3] }
    }
}