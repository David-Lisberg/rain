use std::sync::Arc;

use glam::Vec2;
use noise::{NoiseFn, Perlin};
use rain::engine::{core::RainHandle, mesh::ModelMesh, resource::ARRAY_256X256_ID, texture::Texture, vertex::{ModelVertex, SPRITE_QUAD_INDICES}};
use wgpu::util::DeviceExt;

use crate::game::world::tile::{Tile, TileType};

#[derive(PartialEq)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

pub struct ChunkData {
    pub position: ChunkPosition,
    pub tiles: [Tile; CHUNK_DIM * CHUNK_DIM],
}

pub const CHUNK_DIM: usize = 4; /* indices should be u32s if CHUNK_DIM > 64 */

pub fn generate_chunk(chunk_position: ChunkPosition) -> ChunkData {
    let perlin = Perlin::new(0);

    let scale_factor = 0.07;

    let tiles = std::array::from_fn(|i| {
        let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;
        let mut noise_value = perlin.get([x * scale_factor, y * scale_factor]);
        noise_value = (noise_value + 1.0) / 2.0;

        let _type = match noise_value {
            v if v > 0.6 => TileType::Stone,
            v if v >= 0.3 && v < 0.6 => TileType::Grass,
            v if v < 0.3 => TileType::Dirt,
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