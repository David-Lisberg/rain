use glam::{Mat4, Quat, Vec2, Vec3};
use crate::engine::component::*;

use crate::engine::color::Color;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub layer: u32,
}

impl SpriteInstance {
    pub fn new(
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, pivot: Option<&Pivot2D>, rotation_z: Option<&RotationZ>, rotation: Option<&Rotation>,
        flip: Option<&Flip>, color: Option<&Color>, uv_offset: [f32; 2], uv_scale: [f32; 2], layer: u32,
    ) -> Self {
        let pos = match pos {
            Some(p) => match depth {
                Some(d) => Vec3::new(p.0.x, p.0.y, d.0),
                None => Vec3::new(p.0.x, p.0.y, 0.0),
            }
            None => match depth {
                Some(d) => Vec3::new(0.0, 0.0, d.0),
                None => Vec3::new(0.0, 0.0, 0.0),
            }
        };
        let rotation = match (rotation_z, rotation) {
            (Some(r_z), Some(r)) => r.0 + Quat::from_rotation_z(r_z.0),
            (None, Some(r)) => r.0,
            (Some(r_z), None) => Quat::from_rotation_z(r_z.0),
            (None, None) => Quat::IDENTITY,
        };
        let mut scale = match scale {
            Some(s) => s.0,
            None => Vec2::new(1.0, 1.0),
        };
        if let Some(f) = flip {
            if f.0 {
                scale.x *= -1.0;
            }
            if f.1 {
                scale.y *= -1.0;
            }
        }
        let transform = if let Some(p) = pivot {
            let rotation = Mat4::from_rotation_translation(rotation, pos + p.0.extend(0.0));
            rotation * Mat4::from_scale_rotation_translation(scale.extend(1.0), Quat::IDENTITY, -p.0.extend(0.0))
        } else {
            Mat4::from_scale_rotation_translation(scale.extend(1.0), rotation, pos)
        };
        
        Self {
            transform: transform.to_cols_array_2d(),
            color: Color::rain_color_to_array(color.unwrap_or(&Color::WHITE)),
            uv_offset,
            uv_scale,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 24]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}