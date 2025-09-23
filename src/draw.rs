use crate::color::Color;

pub struct DrawPass {
    pub draw_calls: Vec<DrawCall>,
    pub clear_background_color: Option<Color>
}

impl DrawPass {
    pub fn new() -> Self {
        Self {
            draw_calls: Vec::new(),
            clear_background_color: None,
        }
    }
}

pub enum DrawCall {
    ClearBackground
}