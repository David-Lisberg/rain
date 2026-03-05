use hecs::Entity;
use noise::Perlin;
use rain::engine::{component::{Position2D, Visible}, core::RainHandle, mesh::ModelMesh, resource::ARRAY_256X256_ID, vertex::{ModelVertex, SPRITE_QUAD_INDICES}};
use wgpu::util::DeviceExt;

use crate::{game::{utility::noise::octave_noise_2d, world::{generation::CHUNK_GENERATION_DISTANCE, tile::{Tile, TileType}}}};
use crate::game::player::movement::Player;

#[derive(PartialEq)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

impl From<&Position2D> for ChunkPosition {
    fn from(value: &Position2D) -> Self {
        let x = (value.x / CHUNK_DIM as f32).floor() as i32;
        let y = (value.y / CHUNK_DIM as f32).floor() as i32;
        Self { x, y }
    }
}

pub struct ChunkData {
    pub position: ChunkPosition,
    pub tiles: [Tile; CHUNK_DIM * CHUNK_DIM],
}

pub const CHUNK_DIM: usize = 32; /* indices should be u32s if CHUNK_DIM > 64 */

pub fn generate_chunk(chunk_position: ChunkPosition, perlin: Perlin) -> ChunkData {
    let scale_factor = 0.026;

    let tiles = std::array::from_fn(|i| {
        let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;
        let mut noise_value = octave_noise_2d(x * scale_factor, y * scale_factor, 4, 0.5, &perlin);
        noise_value = (noise_value + 1.0) / 2.0;

        let _type = match noise_value {
            v if v >= 0.85 => TileType::Cobblestone,
            v if v >= 0.65 && v < 0.85 => TileType::Stone,
            v if v >= 0.6 && v < 0.65 => TileType::Dirt,
            v if v >= 0.35 && v < 0.6 => TileType::Grass,
            v if v >= 0.3 && v < 0.35 => TileType::Sand,
            v if v < 0.3 => TileType::Water,
            _ => TileType::Dirt
        };

        Tile { _type }
    });

    ChunkData {
        position: chunk_position,
        tiles,
    }
}

pub fn construct_chunk_mesh(handle: &mut RainHandle, chunk: &ChunkData) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u16> = Vec::new();

    for (i, tile) in chunk.tiles.iter().enumerate() {
        let tile_texture = tile._type.fetch_texture(handle);
        let x = (chunk.position.x * CHUNK_DIM as i32) as f32 + (i % CHUNK_DIM) as f32;
        let y = (chunk.position.y * CHUNK_DIM as i32) as f32 + (i / CHUNK_DIM) as f32;
        
        let vertices = vec![
            ModelVertex { position: [x, y, 0.0], uv: [0.0, 1.0], layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y, 0.0], uv: [1.0, 1.0], layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y + 1.0, 0.0], uv: [1.0, 0.0], layer: tile_texture.index },
            ModelVertex { position: [x, y + 1.0, 0.0], uv: [0.0, 0.0], layer: tile_texture.index },
        ];
        let indices: Vec<u16> = SPRITE_QUAD_INDICES.iter().map(|x| x + i as u16 * 4).collect();
        model_vertices.extend(vertices);
        model_indices.extend(indices);
    }
    
    ModelMesh {
        vertices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_vertex_buffer"),
            contents: bytemuck::cast_slice(&model_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_index_buffer"),
            contents: bytemuck::cast_slice(&model_indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        num_indices: model_indices.len() as u32,
        array_id: ARRAY_256X256_ID,
    }
}

pub fn system_manage_chunks(handle: &mut RainHandle) {
    let mut to_deload: Vec<Entity> = Vec::new();
    let mut to_load: Vec<Entity> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position: ChunkPosition = position.into();

        for (e, (chunk, mesh)) in handle.world.query::<(&ChunkData, Option<&ModelMesh>)>().iter() {
            let radius = CHUNK_GENERATION_DISTANCE / 2;
            if mesh.is_some() {
                

                if chunk.position.x > chunk_position.x + radius ||
                   chunk.position.x < chunk_position.x - radius ||
                   chunk.position.y > chunk_position.y + radius ||
                   chunk.position.y < chunk_position.y - radius {
                    to_deload.push(e);
                }
            } else {
                if chunk.position.x <= chunk_position.x + radius &&
                   chunk.position.x >= chunk_position.x - radius &&
                   chunk.position.y <= chunk_position.y + radius &&
                   chunk.position.y >= chunk_position.y - radius {
                    to_load.push(e);
                }
            }
        }
    }

    for e in to_deload {
        handle.world.remove_one::<ModelMesh>(e).unwrap();
    }
    for e in to_load {
        let (chunk,)= handle.world.remove::<(ChunkData,)>(e).unwrap();
        let mesh = construct_chunk_mesh(handle, &chunk);
        handle.world.spawn((chunk, mesh, Visible));
    }
}