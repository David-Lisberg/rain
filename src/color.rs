pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a
        }
    }

    pub fn rain_color_to_wgpu_color(color: &Self) -> wgpu::Color {
        wgpu::Color {
            r: Self::srgb_to_linear(color.r as f32/ 255.0) as f64,
            g: Self::srgb_to_linear(color.g as f32/ 255.0) as f64,
            b: Self::srgb_to_linear(color.b as f32/ 255.0) as f64,
            a: color.a as f64 / 255.0,
        }
    }

    pub fn rain_color_to_array(color: &Self) -> [f32; 3] {
        [
            Self::srgb_to_linear(color.r as f32/ 255.0),
            Self::srgb_to_linear(color.g as f32/ 255.0),
            Self::srgb_to_linear(color.b as f32/ 255.0),
        ]
    }

    fn srgb_to_linear(color: f32) -> f32 {
        if color <= 0.04045 {
            color / 12.92
        } else {
            ((color + 0.055) / 1.055).powf(2.4)
        }
    }

    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const CYAN: Self = Self { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Self = Self { r: 255, g: 0, b: 255, a: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0, a: 255 };
    pub const ORANGE: Self = Self { r: 255, g: 112, b: 0, a: 255 };
    pub const PURPLE: Self = Self { r: 127, g: 0, b: 127, a: 255 };
    pub const PINK: Self = Self { r: 255, g: 0, b: 186, a: 255 };
    pub const BROWN: Self = Self { r: 97, g: 47, b: 16, a: 255 };
    pub const LIME: Self = Self { r: 50, g: 255, b: 32, a: 255 };
    pub const TEAL: Self = Self { r: 0, g: 127, b: 127, a: 255 };
}