use crate::engine::{color::Color, mesh::Mesh};

pub struct DrawPass {
    pub draw_calls: Vec<DrawCall>,
    pub clear_background_color: Option<Color>
}

impl DrawPass {
    pub fn new(clear_background_color: Option<Color>) -> Self {
        Self {
            draw_calls: Vec::new(),
            clear_background_color,
        }
    }
}

pub enum DrawCall {
    Mesh(Mesh),
}