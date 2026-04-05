use glam::Vec2;

use crate::{State, game::{core::physics::ADJACENT, world::{chunk::{ChunkPosition, position_to_chunk_position}, object::Object}}};

#[derive(Debug, Clone, Copy)]
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

    pub fn aabb_collision(&self, other: &Collider) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

pub fn check_collision_with_object(state: &mut State, collider: &Collider) -> Option<Object> {
    let mut collided: Vec<Object> = Vec::new();
    let chunk_position: ChunkPosition = position_to_chunk_position(collider.x, collider.y);

    for adjacent in ADJACENT {
        let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
        if let Some(chunk) = state.chunks.get(&adjacent_position) {
            for object in chunk.objects.iter() {
                if collider.aabb_collision(&object.collider) {
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
            let distance = (object.collider.center() - collider.center()).length();
            if distance < min_distance {
                min_distance = distance;
                min_index = i;
            }
        }
        Some(collided[min_index])
    }
}