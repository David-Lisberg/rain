use std::{any::Any, sync::Arc};

use lgui::platform::Drawable;

use crate::engine::{color::Color, core::RainHandle, texture::Texture};

impl From<lgui::platform::Color> for Color {
    fn from(value: lgui::platform::Color) -> Self {
        Color::new(value.r, value.g, value.b, value.a)
    }
}

impl From<Color> for lgui::platform::Color {
    fn from(value: Color) -> Self {
        Self { r: value.r, g: value.g, b: value.b, a: value.a }
    }
}

impl lgui::platform::Texture for Texture {
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl Drawable for RainHandle {
    fn lgui_draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: lgui::platform::Color) {
        self.draw_rectangle((x, y, w, h), color.into());
    }

    fn lgui_draw_texture(&mut self, x: f32, y: f32, w: f32, h: f32, texture: std::sync::Arc<dyn lgui::platform::Texture>, tint: lgui::platform::Color) {
        let texture = texture.as_any_arc()
            .downcast::<Texture>()
            .expect("Error: Expected a texture.");
        self.draw_texture((x, y, w, h), texture, tint.into());
    }

    fn lgui_draw_text(&mut self, text: &str, x: f32, y: f32, font_size: u32, color: lgui::platform::Color) {
        self.draw_text(x, y, text, font_size, color.into());
    }

    fn lgui_measure_text(&mut self, text: &str, font_size: u32) -> f32 {
        self.renderer.text_state.measure_text(text, font_size)
    }
}