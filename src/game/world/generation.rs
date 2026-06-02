use rain::engine::core::RainHandle;
use rain::engine::component::*;

use crate::State;
use crate::game::player::movement::Player;
use crate::game::world::chunk::{ChunkPosition, construct_chunk_mesh, generate_chunk};
use crate::game::world::object::reload_object_mesh;

pub const CHUNK_GENERATION_DISTANCE: i32 = 7;

pub fn system_world_generation(handle: &mut RainHandle, state: &mut State) {
    let mut to_generate: Vec<ChunkPosition> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position: ChunkPosition = position.into();

        for i in 0..CHUNK_GENERATION_DISTANCE {
            for j in 0..CHUNK_GENERATION_DISTANCE {
                let adjacent_chunk = ChunkPosition {
                    x: chunk_position.x + i - CHUNK_GENERATION_DISTANCE / 2,
                    y: chunk_position.y + j - CHUNK_GENERATION_DISTANCE / 2,
                };
                if state.chunks.get(&adjacent_chunk).is_none() {
                    to_generate.push(adjacent_chunk);
                }
            }
        }
    }
    for chunk_position in to_generate {
        let chunk = generate_chunk(chunk_position, state);
        // let mesh = construct_chunk_mesh(handle, &chunk);
        handle.world.spawn((chunk_position,));
        state.chunks.insert(chunk_position, chunk);
    }

    // reload_object_mesh(handle, state);
}