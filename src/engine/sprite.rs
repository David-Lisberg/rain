use std::sync::Arc;

use crate::engine::{color::Color, component::*, instance::SpriteInstance, texture::Texture};

#[derive(Clone)]
pub struct SpriteRender {
    pub instance: SpriteInstance,
    pub array_id: u32,
}

impl SpriteRender {
    pub fn new(
        pos: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, rotation: Option<&RotationZ>, color: Option<&Color>, texture: Option<&Arc<Texture>>
    ) -> Self {
        let (instance, array_id) = if let Some(t) = texture {
            (SpriteInstance::new(pos, depth, scale, rotation, color, t.index), t.array_id)
        } else {
            (SpriteInstance::new(pos, depth, scale, rotation, color, 0), 0)
        };
        Self {
            instance,
            array_id,
        }
    }
}