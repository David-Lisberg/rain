use glam::Vec2;
use serde::Deserialize;

use crate::State;
use crate::game::core::physics::ADJACENT_I32;
use crate::game::world::chunk::{ChunkPosition, position_to_chunk_position};
use crate::game::world::object::Object;


#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Collider {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Collider {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_center(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(x - width / 2.0, y - height / 2.0, width, height)
    }

    pub fn add_vec2(&self, rhs: Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            width: self.width,
            height: self.height
        }
    }

    pub fn aabb_collision(&self, other: &Collider) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    pub fn aabb_collision_point(&self, point: &Vec2) -> bool {
        self.x < point.x &&
        self.x + self.width > point.x &&
        self.y < point.y &&
        self.y + self.height > point.y
    }

    pub fn aabb_collision_ray(&self, start: &Vec2, finish: &Vec2) -> bool {
        let inverted_direction = 1.0 / (finish - start).normalize();
        let length = (finish - start).length();

        let t_min_x = (self.x - start.x) * inverted_direction.x;
        let t_max_x = (self.x + self.width - start.x) * inverted_direction.x;
        let t_min_y = (self.y - start.y) * inverted_direction.y;
        let t_max_y = (self.y + self.height - start.y) * inverted_direction.y;

        let t_enter = t_min_x.min(t_max_x).max(t_min_y.min(t_max_y));
        let t_exit  = t_min_x.max(t_max_x).min(t_min_y.max(t_max_y));
        
        t_exit >= 0.0 && t_enter <= t_exit && t_enter <= length
    }

    pub fn aabb_collision_swept(&self, other: &Collider, start: &Vec2, finish: &Vec2) -> bool {
        let new_collider = Collider::new(
            other.x - self.width / 2.0,
            other.y - self.height / 2.0,
            other.width + self.width,
            other.height + self.height
        );
        new_collider.aabb_collision_ray(start, finish)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

pub fn check_collision_with_object(state: &mut State, collider: &Collider) -> Option<Object> {
    let mut collided: Vec<Object> = Vec::new();
    let chunk_position: ChunkPosition = position_to_chunk_position(collider.x, collider.y);

    for adjacent in ADJACENT_I32 {
        let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
        if let Some(chunk) = state.chunks.get(&adjacent_position) {
            for object in chunk.objects.iter() {
                let object_data = state.object_registry.from_id(object.type_id).unwrap();
                if collider.aabb_collision(&object_data.collider.add_vec2(object.position)) {
                    collided.push(object.clone());
                }
            }
        }
    }

    if collided.is_empty() {
        None
    } else {
        let mut min_distance = f32::MAX;
        let mut min_index = 0;

        for (i, object) in collided.iter().enumerate() {
            let object_data = state.object_registry.from_id(object.type_id).unwrap();
            let distance = (object_data.center(object.position) - collider.center()).length();
            if distance < min_distance {
                min_distance = distance;
                min_index = i;
            }
        }
        Some(collided[min_index])
    }
}

pub fn collect_object_colliders(state: &mut State, position: Vec2) -> Vec<Collider> {
    let mut object_colliders: Vec<Collider> = Vec::new();
    let chunk_position = position_to_chunk_position(position.x, position.y);
    for adjacent in ADJACENT_I32 {
        let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
        if let Some(chunk) = state.chunks.get(&adjacent_position) {
            for object in &chunk.objects {
                let object_data = state.object_registry.from_id(object.type_id).unwrap();
                object_colliders.push(object_data.collider.add_vec2(object.position))
            }
        }
    }
    object_colliders
}

pub fn collect_water_colliders(state: &mut State, position: Vec2) -> Vec<Collider> {
    let mut water_colliders: Vec<Collider> = Vec::new();
    let chunk_position = position_to_chunk_position(position.x, position.y);
    for adjacent in ADJACENT_I32 {
        let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
        if let Some(chunk) = state.chunks.get(&adjacent_position) {
            water_colliders.extend(chunk.water_colliders.clone());
        }
    }
    water_colliders
}