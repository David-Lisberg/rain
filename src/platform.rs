use crate::color::Color;
use crate::core::ReignHandle;

impl ReignHandle {
    pub fn clear_background(&mut self, color: Color) {
        self.renderer.draw_pass.clear_background_color = Some(color);
    }

    pub fn draw_rectangle(&mut self, x: f64, y: f64, w: f64, h: f64, color: Color) {
        
    }
}