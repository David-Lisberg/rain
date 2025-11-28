use std::sync::Arc;

use hecs::*;
use glam::*;

use crate::engine::{color::Color, core::RainHandle, instance::SpriteInstance, texture::Texture};

// pub struct SpriteManager {
//     pub sprites: Vec<Entity>,
// }

// impl SpriteManager {
//     pub fn new() -> Self {
//         Self {
//             sprites: Vec::new(),
//         }
//     }
// }

// impl RainHandle {
//     pub fn spawn_sprite(&mut self, position: Vec3, rotation: f32, scale: Vec2, texture: Arc<Texture>, visible: bool) {
//         let transform = SpriteTransform::new(position, scale, rotation);

//         if visible {
//             self.world.spawn((Sprite, SpriteVisible, transform, texture));
//         } else {
//             self.world.spawn((Sprite, transform, texture));
//         }
//     }

//     pub fn spawn_sprite_texture_color(&mut self, position: Vec3, rotation: f32, scale: Vec2, texture: Arc<Texture>, visible: bool, color: Color) {
//         let transform = SpriteTransform::new(position, scale, rotation);

//         if visible {
//             self.world.spawn((Sprite, SpriteVisible, transform, texture, color));
//         } else {
//             self.world.spawn((Sprite, transform, texture, color));
//         }
//     }

//     pub fn spawn_sprite_color(&mut self, position: Vec3, rotation: f32, scale: Vec2, visible: bool, color: Color) {
//         let transform = SpriteTransform::new(position, scale, rotation);

//         if visible {
//             self.world.spawn((Sprite, SpriteVisible, transform, color));
//         } else {
//             self.world.spawn((Sprite, transform, color));
//         }
//     }
// }