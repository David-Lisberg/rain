use std::sync::Arc;

use glam::*;

use crate::color::Color;
use crate::core::RainHandle;
use crate::draw::{DrawCall, DrawPass};
use crate::mesh::Mesh;
use crate::texture::Texture;
use crate::utility::rectangle::Rectangle;
use crate::utility::transform::{framebuffer_to_ndc, rotate_around_pivot};
use crate::vertex::UIVertex;

const INDICES_RECTANGLE: &[u16] = &[
    0, 1, 2,
    1, 3, 2
];

impl RainHandle {
    pub fn clear_background(&mut self, color: Color) {
        self.renderer.draw_pass = DrawPass::new(Some(color));
    }

    pub fn draw_rectangle(&mut self, rect: impl Into<Rectangle>, color: Color) {
        let rect: Rectangle = rect.into();
        let ndc1 = framebuffer_to_ndc((rect.x, rect.y), self.base_width, self.base_height);
        let ndc2 = framebuffer_to_ndc((rect.x + rect.w, rect.y + rect.h), self.base_width, self.base_height);
        let color_array = Color::rain_color_to_array(&color);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            Mesh {
                vertices: vec![
                    UIVertex { position: [ndc1.x, ndc2.y], uv: [0.0, 1.0], layer: 0, color: color_array },
                    UIVertex { position: [ndc2.x, ndc2.y], uv: [1.0, 1.0], layer: 0, color: color_array },
                    UIVertex { position: [ndc1.x, ndc1.y], uv: [0.0, 0.0], layer: 0, color: color_array },
                    UIVertex { position: [ndc2.x, ndc1.y], uv: [1.0, 0.0], layer: 0, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: Arc::clone(self.resource_manager.textures.get("").unwrap()),
            }
        ))
    }

    pub fn draw_rectangle_ex(&mut self, rect: impl Into<Rectangle>, color: Color, degrees: f32, origin: impl Into<Vec2>) {
        let rect: Rectangle = rect.into();
        let origin: Vec2 = origin.into();
        let origin: (f32, f32) = (origin.x + rect.x, origin.y + rect.y);

        let p1 = rotate_around_pivot((rect.x, rect.y + rect.h), origin, degrees);
        let p2 = rotate_around_pivot((rect.x + rect.w, rect.y + rect.h), origin, degrees);
        let p3 = rotate_around_pivot((rect.x, rect.y), origin, degrees);
        let p4 = rotate_around_pivot((rect.x + rect.w, rect.y), origin, degrees);
        
        let p1_ndc = framebuffer_to_ndc(p1, self.base_width, self.base_height);
        let p2_ndc = framebuffer_to_ndc(p2, self.base_width, self.base_height);
        let p3_ndc = framebuffer_to_ndc(p3, self.base_width, self.base_height);
        let p4_ndc = framebuffer_to_ndc(p4, self.base_width, self.base_height);
        let color_array = Color::rain_color_to_array(&color);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            Mesh {
                vertices: vec![
                    UIVertex { position: p1_ndc.into(), uv: [0.0, 1.0], layer: 0, color: color_array },
                    UIVertex { position: p2_ndc.into(), uv: [1.0, 1.0], layer: 0, color: color_array },
                    UIVertex { position: p3_ndc.into(), uv: [0.0, 0.0], layer: 0, color: color_array },
                    UIVertex { position: p4_ndc.into(), uv: [1.0, 0.0], layer: 0, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: Arc::clone(self.resource_manager.textures.get("").unwrap()),
            }
        ))
    }

    pub fn draw_texture(&mut self, rect: impl Into<Rectangle>, texture: Arc<Texture>, tint: Color) {
        let rect: Rectangle = rect.into();
        let ndc1 = framebuffer_to_ndc((rect.x, rect.y), self.base_width, self.base_height);
        let ndc2 = framebuffer_to_ndc((rect.x + rect.w, rect.y + rect.h), self.base_width, self.base_height);
        let color_array = Color::rain_color_to_array(&tint);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            Mesh {
                vertices: vec![
                    UIVertex { position: [ndc1.x, ndc2.y], uv: [0.0, 1.0], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc2.x, ndc2.y], uv: [1.0, 1.0], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc1.x, ndc1.y], uv: [0.0, 0.0], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc2.x, ndc1.y], uv: [1.0, 0.0], layer: texture.index, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: texture,
            }
        ))
    }

    pub fn draw_texture_ex(&mut self, rect: impl Into<Rectangle>, texture: Arc<Texture>, tint: Color, degrees: f32, origin: impl Into<Vec2>) {
        let rect: Rectangle = rect.into();
        let origin: Vec2 = origin.into();
        let origin: (f32, f32) = (origin.x + rect.x, origin.y + rect.y);

        let p1 = rotate_around_pivot((rect.x, rect.y + rect.h), origin, degrees);
        let p2 = rotate_around_pivot((rect.x + rect.w, rect.y + rect.h), origin, degrees);
        let p3 = rotate_around_pivot((rect.x, rect.y), origin, degrees);
        let p4 = rotate_around_pivot((rect.x + rect.w, rect.y), origin, degrees);
        
        let p1_ndc = framebuffer_to_ndc(p1, self.base_width, self.base_height);
        let p2_ndc = framebuffer_to_ndc(p2, self.base_width, self.base_height);
        let p3_ndc = framebuffer_to_ndc(p3, self.base_width, self.base_height);
        let p4_ndc = framebuffer_to_ndc(p4, self.base_width, self.base_height);
        let color_array = Color::rain_color_to_array(&tint);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            Mesh {
                vertices: vec![
                    UIVertex { position: p1_ndc.into(), uv: [0.0, 1.0], layer: texture.index, color: color_array },
                    UIVertex { position: p2_ndc.into(), uv: [1.0, 1.0], layer: texture.index, color: color_array },
                    UIVertex { position: p3_ndc.into(), uv: [0.0, 0.0], layer: texture.index, color: color_array },
                    UIVertex { position: p4_ndc.into(), uv: [1.0, 0.0], layer: texture.index, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: texture,
            }
        ))
    }
}