use std::sync::Arc;

use glam::{Mat4, Quat, Vec2, Vec3};
use crate::engine::component::*;

use crate::engine::color::Color;
use crate::engine::texture::Texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub transform: [[f32; 4]; 4],
    pub color: [f32 ; 4],
    pub layer: u32,
}

impl SpriteInstance {
    pub fn new(
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, rotation: Option<&RotationZ>, color: Option<&Color>, layer: u32,
    ) -> Self {
        let pos = match pos {
            Some(p) => match depth {
                Some(d) => Vec3::new(p.x, p.y, d.0),
                None => Vec3::new(p.x, p.y, 0.0),
            }
            None => match depth {
                Some(d) => Vec3::new(0.0, 0.0, d.0),
                None => Vec3::new(0.0, 0.0, 0.0),
            }
        };
        let rotation = match rotation {
            Some(r) => r.0,
            None => 0.0,
        };
        let scale = match scale {
            Some(s) => s.0,
            None => Vec2::new(1.0, 1.0),
        };
        
        Self {
            transform: (Mat4::from_scale_rotation_translation(scale.extend(1.0), Quat::from_rotation_z(rotation), pos)).to_cols_array_2d(),
            color: Color::rain_color_to_array(color.unwrap_or(&Color::WHITE)),
            layer,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}