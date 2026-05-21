use glam::{FloatExt, Vec2};
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::engine::core::RainHandle;

#[derive(Serialize, Deserialize)]
pub struct AnimationData {
    pub frames: Vec<AnimationFrame>,
    pub repeat: bool,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnimationFrame {
    pub uv_rect: UVRect,
    pub duration: usize,
    pub position: Option<Vec2>,
    pub scale: Option<Vec2>,
    pub rotation: Option<f32>,
}

impl AnimationFrame {
    pub fn new(uv_rect: UVRect, duration: usize) -> Self {
        Self { uv_rect, duration, position: None, scale: None, rotation: None }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct UVRect {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}
pub struct Animation {
    pub name: String,
    pub current_frame: usize,
    pub frame_progress: usize,
    pub uv_rect: UVRect,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Animation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            current_frame: 0,
            frame_progress: 0,
            uv_rect: UVRect::default(),
            position: Vec2::ZERO,
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
        }
    }
}

impl UVRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { offset: [x, y], scale: [width, height] }
    }

    pub fn default() -> Self {
        Self { offset: [0.0, 0.0], scale: [1.0, 1.0] }
    }
}

pub fn system_manage_animations(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();
    let mut animation_ids: Vec<(Entity, String)> = Vec::new();
    for (e, animation) in handle.world.query::<&Animation>().iter() {
        animation_ids.push((e, animation.name.clone()));
    }
    for (e, id) in animation_ids {
        let animation_data = handle.fetch_animation(&id).unwrap();
        if let Ok(animation) = handle.world.query_one_mut::<&mut Animation>(e) {
            let current_frame = &animation_data.frames[animation.current_frame];
            animation.uv_rect = current_frame.uv_rect;

            let s =  1.0 / (current_frame.duration - animation.frame_progress) as f32;
            if let Some(current_position) = current_frame.position {
                animation.position = animation.position.lerp(current_position, s);
            }
            if let Some(current_scale) = current_frame.scale {
                animation.scale = animation.scale.lerp(current_scale, s);
            }
            if let Some(current_rotation) = current_frame.rotation {
                animation.rotation = animation.rotation.lerp(current_rotation, s);
            }
            animation.frame_progress += 1;

            if animation.frame_progress >= current_frame.duration {
                animation.frame_progress = 0;
                animation.current_frame += 1;
            }

            if animation.current_frame >= animation_data.frames.len() {
                if animation_data.repeat {
                    animation.current_frame = 0;
                } else {
                    to_despawn.push(e);
                }
            }
        }
    }

    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}