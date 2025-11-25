use hecs::*;
use glam::*;

use crate::{core::RainHandle, instance::SpriteInstance, texture::Texture};

pub struct Sprite;
pub struct SpriteVisible;
pub struct SpriteCreated;
pub struct SpriteRemoved;
pub struct SpriteTransform {
    position: Vec3,
    scale: Vec2,
    rotation: f32,
}

impl SpriteTransform {
    pub fn to_sprite_instance(&self, layer: u32) -> SpriteInstance {
        SpriteInstance {
            position: self.position.into(),
            scale: self.scale.into(),
            rotation: self.rotation,
            layer
        }
    }
}

pub struct SpriteManager {
    pub sprites: Vec<Entity>,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }
}

impl RainHandle {
    pub fn spawn_sprite(&mut self, position: Vec3, rotation: f32, scale: Vec2, texture: Texture, visible: bool) {
        let transform = SpriteTransform {
            position,
            rotation,
            scale,
        };

        if visible {
            self.world.spawn((Sprite, SpriteVisible, transform, texture));
        } else {
            self.world.spawn((Sprite, transform, texture));
        }
    }
}