use hecs::Entity;
use rain::engine::{component::Position2D, core::RainHandle};

use crate::{State, game::{core::{collision::Collider, physics::ADJACENT_I32}, world::chunk::{ChunkPosition, position_to_chunk_position}}};

pub struct Swimmable;
pub struct Swimming;

pub fn system_swimming(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_swimming: Vec<Entity> = Vec::new();
    let mut to_remove_swimming: Vec<Entity> = Vec::new();

    for (e, (_, position, collider, swimming)) in handle.world.query::<(&Swimmable, &Position2D, &Collider, Option<&Swimming>)>().iter() {
        let mut colliders: Vec<Collider> = Vec::new();
        let chunk_position = position_to_chunk_position(position.0.x, position.0.y);
        for adjacent in ADJACENT_I32 {
            let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
            if let Some(chunk) = state.chunks.get(&adjacent_position) {
                colliders.extend(chunk.water_colliders.clone());
            }
        }

        let mut collided = false;
        let collider_center = collider.center();
        let new_collider = Collider::from_center(collider_center.x, collider_center.y, collider.width / 2.0, collider.height / 2.0);
        for water_collider in colliders {
            if new_collider.aabb_collision(&water_collider) {
                collided = true;
            }
        }
        match (collided, swimming.is_some()) {
            (true, false) => to_add_swimming.push(e),
            (false, true) => to_remove_swimming.push(e),
            _ => {}
        }
    }

    for e in to_add_swimming {
        handle.world.insert_one(e, Swimming).unwrap();
    }
    for e in to_remove_swimming {
        handle.world.remove_one::<Swimming>(e).unwrap();
    }
}