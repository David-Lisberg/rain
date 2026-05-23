use glam::Vec2;

pub enum Direction8 {
    N,
    NW,
    W,
    SW,
    S,
    SE,
    E,
    NE
}

impl Direction8 {
    pub fn from_vec2_8way(value: Vec2) -> Self {
        let angle = value.y.atan2(value.x);
        let section = ((angle / std::f32::consts::TAU) * 8.0).round() as i32;

        match section.rem_euclid(8) {
            0 => Direction8::E,
            1 => Direction8::NE,
            2 => Direction8::N,
            3 => Direction8::NW,
            4 => Direction8::W,
            5 => Direction8::SW,
            6 => Direction8::S,
            _ => Direction8::SE,
        }
    }
}