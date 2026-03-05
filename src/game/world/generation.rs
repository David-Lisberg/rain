use rain::engine::core::RainHandle;
use rain::engine::component::*;

use crate::State;
use crate::game::player::movement::Player;
use crate::game::world::chunk::{ChunkData, ChunkPosition, construct_chunk_mesh, generate_chunk};

pub const CHUNK_GENERATION_DISTANCE: i32 = 5;

pub fn system_world_generation(handle: &mut RainHandle, state: &mut State) {
    let mut to_generate: Vec<ChunkPosition> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position: ChunkPosition = position.into();

        for i in 0..CHUNK_GENERATION_DISTANCE {
            for j in 0..CHUNK_GENERATION_DISTANCE {
                let mut chunk_generated: bool = false;
                let adjacent_chunk = ChunkPosition {
                    x: chunk_position.x + i - CHUNK_GENERATION_DISTANCE / 2,
                    y: chunk_position.y + j - CHUNK_GENERATION_DISTANCE / 2,
                };
                for (_, chunk) in handle.world.query::<&ChunkData>().iter() {
                    if chunk.position == adjacent_chunk {
                        chunk_generated = true;
                    }
                }
                if !chunk_generated {
                    to_generate.push(adjacent_chunk);
                }
            }
        }
    }
    for chunk_position in to_generate {
        let chunk = generate_chunk(chunk_position, state.perlin);
        let mesh = construct_chunk_mesh(handle, &chunk);
        handle.world.spawn((chunk, mesh, Visible));
    }
}