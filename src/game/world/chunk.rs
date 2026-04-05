use glam::Vec2;
use hecs::Entity;
use noise::Perlin;
use rain::engine::{component::{Position2D, Visible}, core::RainHandle, mesh::ModelMesh, resource::ARRAY_256X256_ID, vertex::{ModelVertex, SPRITE_QUAD_INDICES}};
use rand::{RngExt, rngs::ThreadRng};
use wgpu::util::DeviceExt;

use crate::{State, game::{utility::noise::{noise_normalize, octave_noise_2d}, world::{generation::CHUNK_GENERATION_DISTANCE, object::{Object, ObjectType, construct_object_default, reload_object_mesh}, tile::{Tile, TileType}}}};
use crate::game::player::movement::Player;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<&Position2D> for ChunkPosition {
    fn from(value: &Position2D) -> Self {
        let x = (value.0.x / CHUNK_DIM as f32).floor() as i32;
        let y = (value.0.y / CHUNK_DIM as f32).floor() as i32;
        Self { x, y }
    }
}

pub fn position_to_chunk_position(x: f32, y: f32) -> ChunkPosition {
    let chunk_x = (x / CHUNK_DIM as f32).floor() as i32;
    let chunk_y = (y / CHUNK_DIM as f32).floor() as i32;
    ChunkPosition { x: chunk_x, y: chunk_y }
}

pub struct ChunkData {
    pub position: ChunkPosition,
    pub tiles: [Tile; CHUNK_DIM * CHUNK_DIM],
    pub objects: Vec<Object>,
}

pub const CHUNK_DIM: usize = 32; /* indices should be u32s if CHUNK_DIM > 64 */
const NOISE_TILE_SCALE_FACTOR: f64 = 0.026;
const NOISE_OBJECT_SCALE_FACTOR: f64 = 0.017;

pub fn generate_chunk(chunk_position: ChunkPosition, perlin: &Perlin, rng: &mut ThreadRng) -> ChunkData {
    let mut objects: Vec<Object> = Vec::new();

    let tiles = std::array::from_fn(|i| {
        let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;
        let mut noise_value = octave_noise_2d(x * NOISE_TILE_SCALE_FACTOR, y * NOISE_TILE_SCALE_FACTOR, 4, 0.5, &perlin);
        noise_value = noise_normalize(noise_value);

        let _type = match noise_value {
            v if v >= 0.85 => TileType::Cobblestone,
            v if v >= 0.65 && v < 0.85 => TileType::Stone,
            v if v >= 0.6 && v < 0.65 => TileType::Dirt,
            v if v >= 0.35 && v < 0.6 => TileType::Grass,
            v if v >= 0.3 && v < 0.35 => TileType::Sand,
            v if v < 0.3 => TileType::Water,
            _ => TileType::Dirt
        };

        if _type == TileType::Grass {
            let mut noise_value = octave_noise_2d(x * NOISE_OBJECT_SCALE_FACTOR, y * NOISE_OBJECT_SCALE_FACTOR, 2, 0.5, &perlin);
            noise_value = noise_normalize(noise_value);
            noise_value -= rng.random::<f64>();
            let position = Vec2::new(x as f32 + (rng.random::<f32>() - 0.5) / 7.0, y as f32 + (rng.random::<f32>() - 0.5) / 7.0);
            if noise_value > 0.5 {
                objects.push(construct_object_default(ObjectType::Tree1, position));
            } else if noise_value < 0.3 && noise_value > 0.1 {
                objects.push(construct_object_default(ObjectType::Twig, position));
            }
        }

        Tile { _type }
    });

    ChunkData {
        position: chunk_position,
        tiles,
        objects,
    }
}

pub fn construct_chunk_mesh(handle: &mut RainHandle, chunk: &ChunkData) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u16> = Vec::new();

    for (i, tile) in chunk.tiles.iter().enumerate() {
        let tile_texture = tile._type.fetch_texture(&handle.resource_manager);
        let x = (chunk.position.x * CHUNK_DIM as i32) as f32 + (i % CHUNK_DIM) as f32;
        let y = (chunk.position.y * CHUNK_DIM as i32) as f32 + (i / CHUNK_DIM) as f32;
        
        let vertices = vec![
            ModelVertex { position: [x, y, 0.0], uv: [0.0, tile_texture.uv[1]], layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y, 0.0], uv: [tile_texture.uv[0], tile_texture.uv[1]], layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y + 1.0, 0.0], uv: [tile_texture.uv[0], 0.0], layer: tile_texture.index },
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

pub fn system_manage_chunks(handle: &mut RainHandle, state: &mut State) {
    let mut to_deload: Vec<Entity> = Vec::new();
    let mut to_load: Vec<Entity> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position: ChunkPosition = position.into();

        for (e, (chunk, mesh)) in handle.world.query::<(&ChunkPosition, Option<&ModelMesh>)>().iter() {
            let radius = CHUNK_GENERATION_DISTANCE / 2;
            if mesh.is_some() {
                if chunk.x > chunk_position.x + radius ||
                   chunk.x < chunk_position.x - radius ||
                   chunk.y > chunk_position.y + radius ||
                   chunk.y < chunk_position.y - radius {
                    to_deload.push(e);
                }
            } else {
                if chunk.x <= chunk_position.x + radius &&
                   chunk.x >= chunk_position.x - radius &&
                   chunk.y <= chunk_position.y + radius &&
                   chunk.y >= chunk_position.y - radius {
                    to_load.push(e);
                }
            }
        }
    }

    if !to_deload.is_empty() || !to_load.is_empty() {
        reload_object_mesh(handle, state);
    }

    for e in to_deload {
        handle.world.remove_one::<ModelMesh>(e).unwrap();
    }
    for e in to_load {
        let (chunk_position,) = handle.world.remove::<(ChunkPosition,)>(e).unwrap();
        if let Some(chunk) = state.chunks.get(&chunk_position) {
            let mesh = construct_chunk_mesh(handle, chunk);
            handle.world.spawn((chunk_position, mesh, Visible));
        }
    }
}