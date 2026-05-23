use std::sync::Arc;

use glam::Vec2;

use crate::engine::{animation::Animation, color::Color, component::*, instance::SpriteInstance, texture::Texture};

#[derive(Clone)]
pub struct SpriteRender {
    pub instance: SpriteInstance,
    pub array_id: u32,
}

impl SpriteRender {
    pub fn new(
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, pivot: Option<&Pivot2D>, rotation_z: Option<&RotationZ>, rotation: Option<&Rotation>,
        flip: Option<&Flip>, color: Option<&Color>, texture: Option<&Arc<Texture>>, animation: Option<&Animation>,
    ) -> Self {
        let (instance, array_id) = if let Some(t) = texture {
            if let Some(a) = animation {
                let flip_vec = if let Some(f) = flip {
                    match (f.0, f.1) {
                        (true, true) => Vec2::new(-1.0, -1.0),
                        (true, false) => Vec2::new(-1.0, 1.0),
                        (false, false) => Vec2::new(1.0, 1.0),
                        (false, true) => Vec2::new(1.0, -1.0),
                    }
                } else {
                    Vec2::new(1.0, 1.0)
                };
                let uv_scale = [t.uv[0] * a.uv_rect.scale[0], t.uv[1] * a.uv_rect.scale[1]];
                let pos = match pos {
                    Some(p) => Some(&Position2D(p.0 + a.position * flip_vec)),
                    None => Some(&Position2D(a.position * flip_vec)),
                };
                let scale = match scale {
                    Some(s) => Some(&Scale2D(s.0 * a.scale)),
                    None => Some(&Scale2D(a.scale)),
                };
                let pivot = match pivot {
                    Some(p) => Some(&Pivot2D(p.0 + a.pivot * flip_vec)),
                    None => Some(&Pivot2D(a.pivot * flip_vec)),
                };
                let rotation_z = match rotation_z {
                    Some(r) => Some(&RotationZ(r.0 + a.rotation * flip_vec.x)),
                    None => Some(&RotationZ(a.rotation * flip_vec.x)),
                };
                let depth = match depth {
                    Some(r) => Some(&DepthZ(r.0 + a.depth)),
                    None => Some(&DepthZ(a.depth)),
                };
                (SpriteInstance::new(pos, depth, scale, pivot, rotation_z, rotation, flip, color, a.uv_rect.offset, uv_scale, t.index), t.array_id)
            } else {
                (SpriteInstance::new(pos, depth, scale, pivot, rotation_z, rotation, flip, color, [0.0, 0.0], t.uv, t.index), t.array_id)
            }
        } else {
            (SpriteInstance::new(pos, depth, scale, pivot, rotation_z, rotation, flip, color, [0.0, 0.0], [1.0, 1.0], 0), 0)
        };
        Self {
            instance,
            array_id,
        }
    }
}