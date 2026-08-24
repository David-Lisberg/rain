use std::{collections::HashMap, sync::Arc};

use glam::{FloatExt, Vec2};
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::engine::core::RainHandle;

#[derive(Serialize, Deserialize)]
pub struct AnimationData {
    pub frames: Vec<AnimationFrame>,
    pub start: Option<AnimationFrame>,
    pub finish: Option<Vec<AnimationEvent>>,
    pub repeat: bool,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnimationFrame {
    pub uv_rect: UVRect,
    pub duration: usize,
    pub position: Option<Vec2>,
    pub scale: Option<Vec2>,
    pub pivot: Option<Vec2>,
    pub rotation: Option<f32>,
    pub depth: Option<f32>,
    pub events: Option<Vec<AnimationEvent>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AnimationEvent {
    HitBox([f32; 4]), /* collider */
    LockInput,
    RemoveComponent(String),
    AddComponent(String),
}

pub struct AnimationPool {
    pub animations: HashMap<usize, Animation>,
}

impl AnimationPool {
    pub fn new() -> Self {
        Self { animations: HashMap::new() }
    }
}

impl AnimationFrame {
    pub fn new(uv_rect: UVRect, duration: usize) -> Self {
        Self { uv_rect, duration, position: None, scale: None, pivot: None, rotation: None, depth: None, events: None }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct UVRect {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}

#[derive(Clone)]
pub struct Animation {
    pub name: String,
    pub current_frame: usize,
    pub frame_progress: usize,
    pub uv_rect: UVRect,
    pub position: Vec2,
    pub scale: Vec2,
    pub pivot: Vec2,
    pub rotation: f32,
    pub depth: f32,
    pub frame_start: bool,
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
            pivot: Vec2::ZERO,
            rotation: 0.0,
            depth: 0.0,
            frame_start: true,
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
    let mut to_remove: Vec<(Entity, Option<usize>)> = Vec::new();
    let mut animation_ids: Vec<(Entity, String, Option<usize>)> = Vec::new();
    for (e, animation) in handle.world.query::<&Animation>().iter() {
        animation_ids.push((e, animation.name.clone(), None));
    }
    for (e, animation_pool) in handle.world.query::<&AnimationPool>().iter() {
        for (i, animation) in animation_pool.animations.iter() {
            animation_ids.push((e, animation.name.clone(), Some(*i)));
        }
    }
    for (e, id, index) in animation_ids {
        let animation_data = handle.fetch_animation(&id).unwrap();
        if let Some(i) = index {
            if let Ok(animation_pool) = handle.world.query_one_mut::<&mut AnimationPool>(e) {
                process_animation(animation_pool.animations.get_mut(&i).unwrap(), Arc::clone(&animation_data), e, index, &mut to_remove);
            }
        } else {
            if let Ok(animation) = handle.world.query_one_mut::<&mut Animation>(e) {
                process_animation(animation, Arc::clone(&animation_data), e, index, &mut to_remove);
            }
        }
    }

    let mut to_remove_pool: Vec<(Entity, usize)> = to_remove.iter()
        .filter_map(|(e, i)| i.map(|i| (*e, i)))
        .collect();
    to_remove_pool.sort_by_key(|(_, i)| std::cmp::Reverse(*i));
    for (e, i) in to_remove_pool {
        if let Ok(animation_pool) = handle.world.query_one_mut::<&mut AnimationPool>(e) {
            animation_pool.animations.remove(&i);
        }
    }
    for (e, i) in to_remove {
        if i.is_none() {
            handle.world.remove_one::<Animation>(e).unwrap();
        }
    }
}

fn process_animation(animation: &mut Animation, animation_data: Arc<AnimationData>, e: Entity, index: Option<usize>, to_remove: &mut Vec<(Entity, Option<usize>)>) {
    let current_frame = &animation_data.frames[animation.current_frame];
    if !animation.frame_start {
        animation.frame_progress += 1;
    
        if animation.frame_progress >= current_frame.duration {
            animation.frame_progress = 0;
            animation.current_frame += 1;
        }
    
        if animation.current_frame >= animation_data.frames.len() {
            if animation_data.repeat {
                animation.current_frame = 0;
                animation.frame_start = true;
            } else {
                to_remove.push((e, index));
            }
        }
    } else {
        animation.frame_start = false;
    }

    if animation.current_frame == 0 && animation.frame_progress == 0 {
        if let Some(start_frame) = &animation_data.start {
            if let Some(current_position) = start_frame.position {
                animation.position = current_position;
            }
            if let Some(current_scale) = start_frame.scale {
                animation.scale = current_scale;
            }
            if let Some(current_pivot) = start_frame.pivot {
                animation.pivot = current_pivot;
            }
            if let Some(current_rotation) = start_frame.rotation {
                animation.rotation = current_rotation;
            }
            if let Some(current_depth) = start_frame.depth {
                animation.depth = current_depth;
            }
        }
    }

    let previous_frame = if animation.current_frame <= 0 {
        if let Some(start_frame) = &animation_data.start {
            start_frame
        } else {
            &animation_data.frames[animation.current_frame]
        }
    } else {
        &animation_data.frames[animation.current_frame - 1]
    };
    animation.uv_rect = current_frame.uv_rect;

    let s =  animation.frame_progress as f32 / current_frame.duration as f32;
    if let (Some(current_position), Some(previous_position)) = (current_frame.position, previous_frame.position) {
        animation.position = previous_position.lerp(current_position, s);
    }
    if let (Some(current_scale), Some(previous_scale)) = (current_frame.scale, previous_frame.scale) {
        animation.scale = previous_scale.lerp(current_scale, s);
    }
    if let (Some(current_pivot), Some(previous_pivot)) = (current_frame.pivot, previous_frame.pivot) {
        animation.pivot = previous_pivot.lerp(current_pivot, s);
    }
    if let (Some(current_rotation), Some(previous_rotation)) = (current_frame.rotation, previous_frame.rotation) {
        animation.rotation = previous_rotation.lerp(current_rotation, s);
    }
    if let (Some(current_depth), Some(previous_depth)) = (current_frame.depth, previous_frame.depth) {
        animation.depth = previous_depth.lerp(current_depth, s);
    }
}