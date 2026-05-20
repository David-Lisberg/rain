use std::sync::Arc;

use crate::engine::{color::Color, component::*, instance::SpriteInstance, texture::Texture};

#[derive(Clone)]
pub struct SpriteRender {
    pub instance: SpriteInstance,
    pub array_id: u32,
}

impl SpriteRender {
    pub fn new(
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, rotation: Option<&RotationZ>, flip: Option<&Flip>, 
        color: Option<&Color>, texture: Option<&Arc<Texture>>, uv_rect: Option<&UVRect>,
    ) -> Self {
        let (instance, array_id) = if let Some(t) = texture {
            if let Some(uv) = uv_rect {
                let uv_scale = [t.uv[0] * uv.scale[0], t.uv[1] * uv.scale[1]];
                (SpriteInstance::new(pos, depth, scale, rotation, flip, color, uv.offset, uv_scale, t.index), t.array_id)
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