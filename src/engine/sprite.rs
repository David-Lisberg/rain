use std::sync::Arc;

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
        // let (instance, array_id) = if let Some(a) = animation {
        //     let texture = 
        //     let uv_scale = [t.uv[0] * a.uv_rect.scale[0], t.uv[1] * a.uv_rect.scale[1]];
        //     (SpriteInstance::new(pos, depth, scale, rotation, flip, color, a.uv_rect.offset, uv_scale, t.index), t.array_id)
        // } else if let Some(t) = texture {
        //     (SpriteInstance::new(pos, depth, scale, rotation, flip, color, [0.0, 0.0], t.uv, t.index), t.array_id)
        // } else {
        //     (SpriteInstance::new(pos, depth, scale, rotation, flip, color, [0.0, 0.0], [1.0, 1.0], 0), 0)
        // };
        let (instance, array_id) = if let Some(t) = texture {
            if let Some(a) = animation {
                let uv_scale = [t.uv[0] * a.uv_rect.scale[0], t.uv[1] * a.uv_rect.scale[1]];
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