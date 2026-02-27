use glam::Vec2;
use noise::core::perlin::perlin_2d;
use noise::core::simplex::simplex_2d;
use noise::permutationtable::PermutationTable;
use rain::engine::color::Color;
use rain::engine::core::RainHandle;
use rain::engine::component::*;

use noise::{Fbm, NoiseFn, Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};

use crate::Player;
use crate::game::world::chunk::{CHUNK_DIM, ChunkData, ChunkPosition, construct_chunk_mesh, generate_chunk};

pub fn system_world_generation(handle: &mut RainHandle) {
    let mut to_generate: Vec<ChunkPosition> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position = ChunkPosition { x: (position.x / CHUNK_DIM as f32) as i32, y: (position.y / CHUNK_DIM as f32) as i32 };
        let mut chunk_generated: bool = false;
        for (_, chunk) in handle.world.query::<&ChunkData>().iter() {
            if chunk.position == chunk_position {
                chunk_generated = true;
            }
        }
        if !chunk_generated {
            to_generate.push(chunk_position);
        }
    }
    for chunk_position in to_generate {
        let chunk = generate_chunk(chunk_position);
        let mesh = construct_chunk_mesh(handle, &chunk);
        handle.world.spawn((chunk,));
        handle.world.spawn((mesh, Visible));
    }
}