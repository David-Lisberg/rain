use std::sync::Arc;

use glam::*;
use lgui::platform::Drawable;

use crate::engine::color::Color;
use crate::engine::core::RainHandle;
use crate::engine::draw::{DrawCall, DrawPass};
use crate::engine::mesh::UIMesh;
use crate::engine::text::TextInfo;
use crate::engine::texture::Texture;
use crate::engine::utility::rectangle::Rect;
use crate::engine::utility::transform::{framebuffer_to_ndc, rotate_around_pivot};
use crate::engine::vertex::UIVertex;

const INDICES_RECTANGLE: &[u16] = &[
    0, 1, 2,
    1, 3, 2
];

impl RainHandle {
    pub fn clear_background(&mut self, color: Color) {
        self.renderer.draw_pass = DrawPass::new(Some(color));
    }

    pub fn draw_rectangle(&mut self, rect: impl Into<Rect>, color: Color) {
        let rect: Rect = rect.into();
        let ndc1 = framebuffer_to_ndc((rect.x, rect.y), self.renderer.config.width, self.renderer.config.height);
        let ndc2 = framebuffer_to_ndc((rect.x + rect.w, rect.y + rect.h), self.renderer.config.width, self.renderer.config.height);
        let color_array = Color::rain_color_to_array(&color);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            UIMesh {
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

    pub fn draw_rectangle_ex(&mut self, rect: impl Into<Rect>, color: Color, degrees: f32, origin: impl Into<Vec2>) {
        let rect: Rect = rect.into();
        let origin: Vec2 = origin.into();
        let origin: (f32, f32) = (origin.x + rect.x, origin.y + rect.y);

        let p1 = rotate_around_pivot((rect.x, rect.y + rect.h), origin, degrees);
        let p2 = rotate_around_pivot((rect.x + rect.w, rect.y + rect.h), origin, degrees);
        let p3 = rotate_around_pivot((rect.x, rect.y), origin, degrees);
        let p4 = rotate_around_pivot((rect.x + rect.w, rect.y), origin, degrees);
        
        let p1_ndc = framebuffer_to_ndc(p1, self.renderer.config.width, self.renderer.config.height);
        let p2_ndc = framebuffer_to_ndc(p2, self.renderer.config.width, self.renderer.config.height);
        let p3_ndc = framebuffer_to_ndc(p3, self.renderer.config.width, self.renderer.config.height);
        let p4_ndc = framebuffer_to_ndc(p4, self.renderer.config.width, self.renderer.config.height);
        let color_array = Color::rain_color_to_array(&color);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            UIMesh {
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

    pub fn draw_texture(&mut self, rect: impl Into<Rect>, texture: Arc<Texture>, tint: Color) {
        let rect: Rect = rect.into();
        let ndc1 = framebuffer_to_ndc((rect.x, rect.y), self.renderer.config.width, self.renderer.config.height);
        let ndc2 = framebuffer_to_ndc((rect.x + rect.w, rect.y + rect.h), self.renderer.config.width, self.renderer.config.height);
        let color_array = Color::rain_color_to_array(&tint);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            UIMesh {
                vertices: vec![
                    UIVertex { position: [ndc1.x, ndc2.y], uv: [0.0, texture.uv[1]], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc2.x, ndc2.y], uv: [texture.uv[0], texture.uv[1]], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc1.x, ndc1.y], uv: [0.0, 0.0], layer: texture.index, color: color_array },
                    UIVertex { position: [ndc2.x, ndc1.y], uv: [texture.uv[0], 0.0], layer: texture.index, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: texture,
            }
        ))
    }

    pub fn draw_texture_ex(&mut self, rect: impl Into<Rect>, texture: Arc<Texture>, tint: Color, degrees: f32, origin: impl Into<Vec2>) {
        let rect: Rect = rect.into();
        let origin: Vec2 = origin.into();
        let origin: (f32, f32) = (origin.x + rect.x, origin.y + rect.y);

        let p1 = rotate_around_pivot((rect.x, rect.y + rect.h), origin, degrees);
        let p2 = rotate_around_pivot((rect.x + rect.w, rect.y + rect.h), origin, degrees);
        let p3 = rotate_around_pivot((rect.x, rect.y), origin, degrees);
        let p4 = rotate_around_pivot((rect.x + rect.w, rect.y), origin, degrees);
        
        let p1_ndc = framebuffer_to_ndc(p1, self.renderer.config.width, self.renderer.config.height);
        let p2_ndc = framebuffer_to_ndc(p2, self.renderer.config.width, self.renderer.config.height);
        let p3_ndc = framebuffer_to_ndc(p3, self.renderer.config.width, self.renderer.config.height);
        let p4_ndc = framebuffer_to_ndc(p4, self.renderer.config.width, self.renderer.config.height);
        let color_array = Color::rain_color_to_array(&tint);
        self.renderer.draw_pass.draw_calls.push(DrawCall::Mesh(
            UIMesh {
                vertices: vec![
                    UIVertex { position: p1_ndc.into(), uv: [0.0, texture.uv[1]], layer: texture.index, color: color_array },
                    UIVertex { position: p2_ndc.into(), uv: [texture.uv[0], texture.uv[1]], layer: texture.index, color: color_array },
                    UIVertex { position: p3_ndc.into(), uv: [0.0, 0.0], layer: texture.index, color: color_array },
                    UIVertex { position: p4_ndc.into(), uv: [texture.uv[0], 0.0], layer: texture.index, color: color_array },
                ],
                indices: INDICES_RECTANGLE.to_vec(),
                material: texture,
            }
        ))
    }

    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, font_size: u32, color: Color) {
        let buffer_index = self.renderer.text_state.buffer_pool.add_text(text, font_size, &mut self.renderer.text_state.font_system);

        let text_info = TextInfo {
            buffer_index,
            x,
            y,
            color: Color::rain_color_to_glyphon_color(&color),
        };
        self.renderer.text_state.to_draw.push(text_info);
    }

    pub fn draw_text_bounded(&mut self, text: &String, font_size: u32, color: Color, width: f32, x: f32, mut y: f32) {
        let mut current = String::new();
        for word in text.split_whitespace() {
            let mut temp = current.clone();
            temp.push_str(word);
            if self.measure_text(&temp, font_size) > width {
                self.draw_text(x, y, &current, font_size, color);
                y += font_size as f32 * 1.3;
                current.clear();
            }
            current.push_str(word);
            current.push(' ');
        }
        self.draw_text(x, y, &current, font_size, color);
    }

    pub fn measure_text(&mut self, text: &str, font_size: u32) -> f32 {
        self.lgui_measure_text(text, font_size)
    }

    pub fn measure_text_height(&mut self, text: &String, font_size: u32, width: f32) -> f32 {
        let mut current = String::new();
        let mut height = 0.0;
        for word in text.split_whitespace() {
            let mut temp = current.clone();
            temp.push_str(word);
            if self.measure_text(&temp, font_size) > width {
                height += font_size as f32 * 1.3;
                current.clear();
            }
            current.push_str(word);
            current.push(' ');
        }
        height + font_size as f32 * 1.3
    }
}