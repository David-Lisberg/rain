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

#[derive(Debug, PartialEq)]
pub enum Direction4 {
    N,
    W,
    S,
    E,
}

impl Direction8 {
    pub fn from_vec2(value: Vec2) -> Self {
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

impl Direction4 {
    pub fn from_vec2(value: Vec2) -> Self {
        if value.y > 0.8 {
            Direction4::N
        } else if value.y < -0.8 {
            Direction4::S
        } else if value.x.is_sign_negative() {
            Direction4::W
        } else {
            Direction4::E
        }
    }
}