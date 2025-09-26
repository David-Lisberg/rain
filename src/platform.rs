use crate::color::Color;
use crate::core::RainHandle;
use crate::draw::DrawPass;

impl RainHandle {
    pub fn clear_background(&mut self, color: Color) {
        self.renderer.draw_pass = DrawPass::new(Some(color));
    }

    pub fn draw_rectangle(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        
    }
}