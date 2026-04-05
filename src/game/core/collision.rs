use rain::engine::{core::RainHandle, mesh::ModelMesh};

use crate::game::world::{chunk::{ChunkData, ChunkPosition, position_to_chunk_position}, object::Object};

#[derive(Debug)]
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

    pub fn aabb_collision(&self, other: &Collider) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
}

fn check_collision_with_object(handle: &mut RainHandle, collider: &Collider) -> Vec<Object> {
    let collided: Vec<Object> = Vec::new();
    let chunk_position: ChunkPosition = position_to_chunk_position(collider.x, collider.y);

    for (_, (chunk, _)) in handle.world.query::<(&ChunkData, &ModelMesh)>().iter() {
        if chunk.position.x <= chunk_position.x + 1 &&
            chunk.position.x >= chunk_position.x - 1 &&
            chunk.position.y <= chunk_position.y + 1 &&
            chunk.position.y >= chunk_position.y - 1 {
            for object in &chunk.objects {
                if object.collidable && collider.aabb_collision(&object.collider) {
                    
                }
            }
        }
    }

    collided
}