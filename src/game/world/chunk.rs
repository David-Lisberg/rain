use std::collections::{HashMap, VecDeque};

use glam::{IVec2, Vec2};
use hecs::Entity;
use rain::engine::{color::Color, component::{Position2D, Visible}, core::RainHandle, mesh::ModelMesh, resource::ARRAY_256X256_ID, vertex::{ModelVertex, SPRITE_QUAD_INDICES}};
use rand::RngExt;
use wgpu::util::DeviceExt;

use crate::{State, game::{core::collision::Collider, utility::noise::{noise_normalize, octave_noise_2d}, world::{generation::CHUNK_GENERATION_DISTANCE, object::{Object, construct_object_default, reload_object_mesh}, tile::{Tile, TileType}}}};
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
    pub water_colliders: Vec<Collider>,
}

pub const CHUNK_DIM: usize = 32; /* indices should be u32s if CHUNK_DIM > 64 */
const NOISE_TILE_SCALE_FACTOR: f64 = 0.0123;
const NOISE_OBJECT_SCALE_FACTOR: f64 = 0.037;
const NOISE_DENSITY_SCALE_FACTOR: f64 = 0.0243;
const NOISE_WATER_BANK_SCALE_FACTOR: f64 = 0.035;
const ADJACENT_TILE: [IVec2; 4] = [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)];

pub fn generate_chunk(chunk_position: ChunkPosition, state: &mut State) -> ChunkData {
    let mut objects: Vec<Object> = Vec::new();

    let mut tiles = std::array::from_fn(|i| {
        let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;
        let mut noise_value = octave_noise_2d(x * NOISE_TILE_SCALE_FACTOR, y * NOISE_TILE_SCALE_FACTOR, 6, 0.5, &state.perlin);
        noise_value = noise_normalize(noise_value);

        let tile_type = {
            let mut tile_type: Option<TileType> = None;
            for ((low, high), tile) in state.world_gen_config.tile_rule {
                if noise_value >= *low && noise_value < *high {
                    tile_type = Some(tile.clone());
                }
            }
            tile_type.unwrap_or(state.world_gen_config.default_tile.clone())
        };

        Tile { _type: tile_type }
    });


    let mut water_bank_queue: VecDeque<usize> = VecDeque::new();
    let mut water_distance: HashMap<usize, f32> = HashMap::new();

    for (i, tile) in tiles.iter().enumerate() {
        if tile._type == TileType::Water {
            water_bank_queue.push_back(i);
            water_distance.insert(i, 0.0);
        }
    }

    while let Some(index) = water_bank_queue.pop_front() {
        let tile_position = IVec2::new((index % CHUNK_DIM) as i32, (index / CHUNK_DIM) as i32);

        for adjacent in ADJACENT_TILE {
            let neighbor_position = tile_position + adjacent;

            if neighbor_position.x < 0 || neighbor_position.x >= CHUNK_DIM as i32 ||
               neighbor_position.y < 0 || neighbor_position.y >= CHUNK_DIM as i32 {
                continue;
            }

            let neighbor = neighbor_position.y as usize * CHUNK_DIM + neighbor_position.x as usize;
            if !water_distance.contains_key(&neighbor) {
                water_distance.insert(neighbor, water_distance[&index] + 1.0);
                water_bank_queue.push_back(neighbor);
            }
        }
    }

    for (index, distance) in std::mem::take(&mut water_distance) {
        if distance <= 5.0 && distance > 0.0 {
            let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (index % CHUNK_DIM) as f64;
            let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (index / CHUNK_DIM) as f64;
            let mut noise_value = octave_noise_2d(x * NOISE_WATER_BANK_SCALE_FACTOR, y * NOISE_WATER_BANK_SCALE_FACTOR, 2, 0.5, &state.perlin);
            noise_value = noise_normalize(noise_value) + (5.0 - distance as f64) / 11.0;
            
            match noise_value {
                v if v >= 0.85 => tiles[index] = Tile { _type: TileType::Clay },
                v if v >= 0.5 && v < 0.85 && tiles[index]._type == TileType::Grass => tiles[index] = Tile { _type: TileType::Mud },
                _ => {}
            }
        }
    }

    for (i, tile) in tiles.iter().enumerate() {
        let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;

        let mut object: Option<Object> = None;

        let mut noise_value = octave_noise_2d(x * NOISE_OBJECT_SCALE_FACTOR, y * NOISE_OBJECT_SCALE_FACTOR, 2, 0.5, &state.perlin);
        let mut noise_density = octave_noise_2d(x * NOISE_DENSITY_SCALE_FACTOR, y * NOISE_DENSITY_SCALE_FACTOR, 2, 0.5, &state.perlin);
        noise_value = noise_normalize(noise_value);
        noise_density = noise_density * 0.25 + 0.6;

        let position = Vec2::new(x as f32 + (state.rng.random::<f32>() - 0.5) / 7.0, y as f32 + (state.rng.random::<f32>() - 0.5) / 7.0);

        for ((low, high), chance, tile_types, object_type) in state.world_gen_config.object_rule {
            let random_value = state.rng.random::<f64>() * noise_density;
            if noise_value >= *low && noise_value < *high && random_value <= *chance {
                for tile_type in tile_types.iter() {
                    if tile._type == *tile_type {
                        object = Some(construct_object_default(*object_type, position));
                        break;
                    }
                }
            }
            if object.is_some() {
                break;
            }
        }
        if let Some(o) = object {
            objects.push(o);
        }
    }

    let mut processed: [bool; CHUNK_DIM * CHUNK_DIM] = std::array::from_fn(|_| false);
    let mut index: usize = 0;
    let mut water_colliders: Vec<Collider> = Vec::new();

    loop {
        while index < tiles.len() && (tiles[index]._type != TileType::Water || processed[index]) {
            index += 1;
        }

        let start = index;
        if index >= tiles.len() {
            break;
        }

        index += 1;
        while index < tiles.len() && index % CHUNK_DIM > start % CHUNK_DIM && tiles[index]._type == TileType::Water && !processed[index] {
            index += 1;
        }
        let width = index - start;
        let mut height = 1;

        loop {
            let mut valid = true;
            for offset in 0..width {
                let i = start + height * CHUNK_DIM + offset;
                if i >= tiles.len() || tiles[i]._type != TileType::Water || processed[i] {
                    valid = false;
                    break;
                }
            }

            if !valid {
                break;
            }
            height += 1;
        }

        for x in 0..width {
            for y in 0..height {
                let i = start + y * CHUNK_DIM + x;
                processed[i] = true;
            }
        }

        let x = (chunk_position.x * CHUNK_DIM as i32) as f32 + (start % CHUNK_DIM) as f32;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f32 + (start / CHUNK_DIM) as f32;
        let collider = Collider::new(x, y, width as f32, height as f32);
        water_colliders.push(collider);
    }
    
    ChunkData {
        position: chunk_position,
        tiles,
        objects,
        water_colliders,
    }
}

pub fn construct_chunk_mesh(handle: &mut RainHandle, chunk: &ChunkData) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u16> = Vec::new();

    let color = Color::rain_color_to_array(&Color::WHITE);

    for (i, tile) in chunk.tiles.iter().enumerate() {
        let tile_texture = tile._type.fetch_texture(&handle.resource_manager);
        let x = (chunk.position.x * CHUNK_DIM as i32) as f32 + (i % CHUNK_DIM) as f32;
        let y = (chunk.position.y * CHUNK_DIM as i32) as f32 + (i / CHUNK_DIM) as f32;
        
        let vertices = vec![
            ModelVertex { position: [x, y, 0.0], uv: [0.0, tile_texture.uv[1]], color, layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y, 0.0], uv: [tile_texture.uv[0], tile_texture.uv[1]], color, layer: tile_texture.index },
            ModelVertex { position: [x + 1.0, y + 1.0, 0.0], uv: [tile_texture.uv[0], 0.0], color, layer: tile_texture.index },
            ModelVertex { position: [x, y + 1.0, 0.0], uv: [0.0, 0.0], color, layer: tile_texture.index },
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