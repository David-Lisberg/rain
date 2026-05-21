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
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, rotation: Option<&RotationZ>, flip: Option<&Flip>, 
        color: Option<&Color>, texture: Option<&Arc<Texture>>, animation: Option<&Animation>,
    ) -> Self {
        let (instance, array_id) = if let Some(t) = texture {
            if let Some(a) = animation {
                let uv_scale = [t.uv[0] * a.uv_rect.scale[0], t.uv[1] * a.uv_rect.scale[1]];
                let pos = match pos {
                    Some(p) => Some(&Position2D(p.0 + a.position)),
                    None => None,
                };
                let scale = match scale {
                    Some(s) => Some(&Scale2D(Vec2::new(s.0.x * a.scale.x, s.0.y * a.scale.y))),
                    None => None,
                };
                let rotation = match rotation {
                    Some(r) => Some(&RotationZ(r.0 + a.rotation)),
                    None => None,
                };
                (SpriteInstance::new(pos, depth, scale, rotation, flip, color, a.uv_rect.offset, uv_scale, t.index), t.array_id)
            } else {
                (SpriteInstance::new(pos, depth, scale, rotation, flip, color, [0.0, 0.0], t.uv, t.index), t.array_id)
            }
        } else {
            (SpriteInstance::new(pos, depth, scale, rotation, flip, color, [0.0, 0.0], [1.0, 1.0], 0), 0)
        };
        Self {
            instance,
            array_id,
        }
    }
}