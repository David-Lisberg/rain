pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y, 
            w, 
            h, 
        }
    }
}

impl From<(f32, f32, f32, f32)> for Rectangle {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Rectangle { x: value.0, y: value.1, w: value.2, h: value.3 }
    }
}

impl From<[f32; 4]> for Rectangle {
    fn from(value: [f32; 4]) -> Self {
        Rectangle { x: value[0], y: value[1], w: value[2], h: value[3] }
    }
}