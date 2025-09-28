use crate::color::Color;
use crate::core::RainHandle;
use crate::draw::{DrawCall, DrawPass};
use crate::mesh::Mesh;
use crate::utility::transform::framebuffer_to_ndc;
use crate::vertex::Vertex;

const INDICES_RECTANGLE: &[u16] = &[
    0, 1, 2,
    1, 3, 2
];

impl RainHandle {
    pub fn clear_background(&mut self, color: Color) {
        self.renderer.draw_pass = DrawPass::new(Some(color));
    }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let (ndc_x, ndc_y) = framebuffer_to_ndc(x, y, self.renderer.config.width, self.renderer.config.height);
        let (ndc_w, ndc_h) = framebuffer_to_ndc(x + w, y + h, self.renderer.config.width, self.renderer.config.height);
        let color_array = Color::rain_color_to_array(&color);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            Mesh {
                vertices: vec![
                    Vertex { position: [ndc_x, ndc_y, 0.0], color: color_array },
                    Vertex { position: [ndc_w, ndc_y, 0.0], color: color_array },
                    Vertex { position: [ndc_x, ndc_h, 0.0], color: color_array },
                    Vertex { position: [ndc_w, ndc_h, 0.0], color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
            }
        ))
    }
}