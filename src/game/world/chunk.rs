use std::collections::{HashMap, VecDeque};

use glam::{IVec2, Vec2};
use hecs::Entity;
use rain::engine::color::Color;
use rain::engine::component::*;
use rain::engine::core::RainHandle;
use rain::engine::mesh::ModelMesh;
use rain::engine::resource::ARRAY_256X256_ID;
use rain::engine::vertex::{ModelVertex, QUAD_INDICES};
use rand::RngExt;
use wgpu::util::DeviceExt;

use crate::State;
use crate::game::core::collision::Collider;
use crate::game::core::physics::ADJACENT_I32;
use crate::game::utility::noise::{noise_normalize, octave_noise_2d};
use crate::game::world::config::BiomeType;
use crate::game::world::generation::CHUNK_GENERATION_DISTANCE;
use crate::game::world::tile::{Tile, TileType};
use crate::game::world::object::{Object, construct_object_default, reload_object_mesh};
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
    pub tiles: [[Tile; CHUNK_DIM]; CHUNK_DIM],
    pub objects: Vec<Object>,
    pub water_colliders: Vec<Collider>,
}

pub struct ChunkInfo {
    position: ChunkPosition,
    data: [[(f64, BiomeType); CHUNK_DIM]; CHUNK_DIM],
}

pub const CHUNK_DIM: usize = 32; /* indices should be u32s if CHUNK_DIM > 64 */
const NOISE_BIOME_SCALE_FACTOR: f64 = 0.007346;
const NOISE_TILE_SCALE_FACTOR: f64 = 0.0123;
const NOISE_OBJECT_SCALE_FACTOR: f64 = 0.037;
const NOISE_DENSITY_SCALE_FACTOR: f64 = 0.0243;
const NOISE_WATER_BANK_SCALE_FACTOR: f64 = 0.035;
const ADJACENT_TILE: [IVec2; 4] = [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)];

const SCALE_CONTINENTALNESS: f64 = 0.007346;
const SCALE_EROSION: f64 = 0.02013;
const SCALE_RIVER: f64 = 0.0049;

const RIVER_THRESHOLD: f64 = 0.11;

const MAX_HEIGHT: f64 = 3.0;

const SPLINE_CONTINENTALNESS: &[(f64, f64)] = &[(-1.0, 0.0), (-0.35, 0.2), (-0.3, 1.1), (-0.15, 1.3), (-0.1, 1.6), (1.0, 2.0)];
const SPLINE_EROSION: &[(f64, f64)] = &[(-1.0, 1.0), (0.0, 0.2), (1.0, 0.1)];

pub fn generate_chunk_info(state: &mut State, chunk_position: ChunkPosition) -> ChunkInfo {
    let data = std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + i as f64;
            let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + j as f64;
    
            let continentalness_noise = octave_noise_2d(x * SCALE_CONTINENTALNESS, y * SCALE_CONTINENTALNESS, 5, 0.5, &state.perlin[0]);
            let erosion_noise = octave_noise_2d(x * SCALE_EROSION, y * SCALE_EROSION, 3, 0.5, &state.perlin[1]);
    
            let continentalness = evaluate_spline(SPLINE_CONTINENTALNESS, continentalness_noise);
            let erosion = evaluate_spline(SPLINE_EROSION, erosion_noise);
            let mut height = continentalness + erosion;
            let mut biome = match height {
                v if v >= 0.0 && v < 1.0 => BiomeType::Ocean,
                v if v >= 1.0 && v < 1.4 => BiomeType::Coast,
                _ => BiomeType::Forest,
            };

            let river_noise = octave_noise_2d(x * SCALE_RIVER, y * SCALE_RIVER, 4, 0.5, &state.perlin[2]);

            if river_noise.abs() < RIVER_THRESHOLD {
                height = 0.0f64.max(height + (RIVER_THRESHOLD - river_noise.abs()) * -12.5);
                if biome != BiomeType::Ocean && height < 1.0 {
                    biome = BiomeType::River;
                }
            }

            (height, biome)
        })
    });
    ChunkInfo { position: chunk_position, data }
}

fn evaluate_spline(spline: &[(f64, f64)], input: f64) -> f64 {
    let upper = spline.iter().position(|&(x, _)| x >= input).unwrap_or(spline.len() - 1);
    let lower = upper.saturating_sub(1);
    if lower == upper {
        return spline[lower].1;
    }
    let (x0, y0) = spline[lower];
    let (x1, y1) = spline[upper];
    let t = (input - x0) / (x1 - x0);
    y0 + (y1 - y0) * t
}

pub fn generate_chunk(chunk_position: ChunkPosition, state: &mut State) -> ChunkData {
    let mut objects: Vec<Object> = Vec::new();
    let chunk_info = generate_chunk_info(state, chunk_position);

    let mut tiles: [[Tile; CHUNK_DIM]; CHUNK_DIM] = std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let (height, biome_type) = &chunk_info.data[i][j];
            let tile_type = {
                let mut tile_type: Option<TileType> = None;
                for biome_rule in &state.world_gen_config {
                    if biome_rule._type == *biome_type {
                        for ((low, high), tile) in &biome_rule.tile_rule {
                            if *height >= *low && *height < *high {
                                tile_type = Some(tile.clone());
                            }
                        }
                        tile_type = Some(tile_type.unwrap_or(biome_rule.default_tile.clone()));
                    }
                }
                tile_type.unwrap()
            };

            Tile { _type: tile_type }
        })
    });

    // let mut tiles = std::array::from_fn(|i| {
    //     let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + (i % CHUNK_DIM) as f64;
    //     let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + (i / CHUNK_DIM) as f64;
    //     let mut noise_value = octave_noise_2d(x * NOISE_TILE_SCALE_FACTOR, y * NOISE_TILE_SCALE_FACTOR, 6, 0.5, &state.perlin[0]);
    //     noise_value = noise_normalize(noise_value);
    //     let mut biome_noise = octave_noise_2d(x * NOISE_BIOME_SCALE_FACTOR, y * NOISE_BIOME_SCALE_FACTOR, 5, 0.5, &state.perlin[1]);
    //     biome_noise = noise_normalize(biome_noise);
        
    //     let biome_type = match biome_noise {
    //         v if v > 0.45 => BiomeType::Forest,
    //         _ => BiomeType::None,
    //     };

    //     let tile_type = {
    //         let mut tile_type: Option<TileType> = None;
    //         for biome_rule in &state.world_gen_config {
    //             if biome_rule._type == biome_type {
    //                 for ((low, high), tile) in &biome_rule.tile_rule {
    //                     if noise_value >= *low && noise_value < *high {
    //                         tile_type = Some(tile.clone());
    //                     }
    //                 }
    //                 tile_type = Some(tile_type.unwrap_or(biome_rule.default_tile.clone()));
    //             }
    //         }
    //         tile_type.unwrap()
    //     };

    //     Tile { _type: tile_type }
    // });


    let mut water_bank_queue: VecDeque<IVec2> = VecDeque::new();
    let mut water_distance: HashMap<IVec2, f32> = HashMap::new();

    for i in 0..CHUNK_DIM {
        for j in 0..CHUNK_DIM {
            if tiles[i][j]._type == TileType::Water {
                water_bank_queue.push_back(IVec2::new(i as i32, j as i32));
                water_distance.insert(IVec2::new(i as i32, j as i32), 0.0);
            }
        }
    }

    while let Some(tile_position) = water_bank_queue.pop_front() {
        for adjacent in ADJACENT_TILE {
            let neighbor_position = tile_position + adjacent;

            if neighbor_position.x < 0 || neighbor_position.x >= CHUNK_DIM as i32 ||
               neighbor_position.y < 0 || neighbor_position.y >= CHUNK_DIM as i32 {
                continue;
            }

            if !water_distance.contains_key(&neighbor_position) {
                water_distance.insert(neighbor_position, water_distance[&tile_position] + 1.0);
                water_bank_queue.push_back(neighbor_position);
            }
        }
    }

    for (tile_position, distance) in std::mem::take(&mut water_distance) {
        if distance <= 5.0 && distance > 0.0 {
            let x = ((chunk_position.x * CHUNK_DIM as i32) + tile_position.x) as f64;
            let y = ((chunk_position.y * CHUNK_DIM as i32) + tile_position.y) as f64;
            let mut noise_value = octave_noise_2d(x * NOISE_WATER_BANK_SCALE_FACTOR, y * NOISE_WATER_BANK_SCALE_FACTOR, 2, 0.5, &state.perlin[1]);
            noise_value = noise_normalize(noise_value) + (5.0 - distance as f64) / 11.0;
            let tile = &mut tiles[tile_position.x as usize][tile_position.y as usize];
            
            match noise_value {
                v if v >= 0.85 => *tile = Tile { _type: TileType::Clay },
                v if v >= 0.5 && v < 0.85 && tile._type == TileType::Grass => *tile = Tile { _type: TileType::Mud },
                _ => {}
            }
        }
    }

    for i in 0..CHUNK_DIM {
        for j in 0..CHUNK_DIM {
            let x = (chunk_position.x * CHUNK_DIM as i32) as f64 + i as f64;
            let y = (chunk_position.y * CHUNK_DIM as i32) as f64 + j as f64;

            let mut object: Option<Object> = None;

            let mut noise_value = octave_noise_2d(x * NOISE_OBJECT_SCALE_FACTOR, y * NOISE_OBJECT_SCALE_FACTOR, 2, 0.5, &state.perlin[0]);
            let mut noise_density = octave_noise_2d(x * NOISE_DENSITY_SCALE_FACTOR, y * NOISE_DENSITY_SCALE_FACTOR, 2, 0.5, &state.perlin[0]);
            noise_value = noise_normalize(noise_value);
            noise_density = noise_density * 0.25 + 0.6;

            let position = Vec2::new(x as f32 + (state.rng.random::<f32>() - 0.5) / 7.0, y as f32 + (state.rng.random::<f32>() - 0.5) / 7.0);

            for biome_rule in &state.world_gen_config {
                for ((low, high), chance, tile_types, object_type) in &biome_rule.object_rule {
                    let random_value = state.rng.random::<f64>() * noise_density;
                    if noise_value >= *low && noise_value < *high && random_value <= *chance {
                        for tile_type in tile_types.iter() {
                            if tiles[i][j]._type == *tile_type {
                                object = Some(construct_object_default(*object_type, position));
                                break;
                            }
                        }
                    }
                    if object.is_some() {
                        break;
                    }
                }
            }
            if let Some(o) = object {
                objects.push(o);
            }
        }
    }

    let mut processed: [[bool; CHUNK_DIM]; CHUNK_DIM] = std::array::from_fn(|_| std::array::from_fn(|_| false));
    let mut index: (usize, usize) = (0, 0);
    let mut water_colliders: Vec<Collider> = Vec::new();

    loop {
        while index.1 < tiles.len() && (tiles[index.0][index.1]._type != TileType::Water || processed[index.0][index.1]) {
            index = increment(index, tiles.len());
        }

        let start = index;
        if index.1 >= tiles.len() {
            break;
        }

        index = increment(index, tiles.len());
        while index.1 < tiles.len() && index.0 > start.0 && tiles[index.0][index.1]._type == TileType::Water && !processed[index.0][index.1] {
            index = increment(index, tiles.len());
        }
        let width = (index.0 as i32 - start.0 as i32 + (index.1 - start.1) as i32 * tiles.len() as i32) as usize;
        let mut height = 1;

        loop {
            let mut valid = true;
            for offset in 0..width {
                let i = (start.0 + offset, start.1 + height);
                if i.1 >= tiles.len() || tiles[i.0][i.1]._type != TileType::Water || processed[i.0][i.1] {
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
                let i = (start.0 + x, start.1 + y);
                processed[i.0][i.1] = true;
            }
        }

        let x = (chunk_position.x * CHUNK_DIM as i32) as f32 + start.0 as f32;
        let y = (chunk_position.y * CHUNK_DIM as i32) as f32 + start.1 as f32;
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

fn increment(mut value: (usize, usize), max: usize) -> (usize, usize) {
    value.0 += 1;
    if value.0 >= max {
        value.0 = 0;
        value.1 += 1;
    }
    value
}

pub fn construct_chunk_mesh(handle: &mut RainHandle, chunk: &ChunkData) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u32> = Vec::new();

    let color = Color::rain_color_to_array(&Color::WHITE);

    for i in 0..CHUNK_DIM {
        for j in 0..CHUNK_DIM {
            let tile_texture = chunk.tiles[i][j]._type.fetch_texture(&handle.resource_manager);
            let x = (chunk.position.x * CHUNK_DIM as i32) as f32 + i as f32;
            let y = (chunk.position.y * CHUNK_DIM as i32) as f32 + j as f32;
            
            let vertices = vec![
                ModelVertex { position: [x, y, 0.0], uv: [0.0, tile_texture.uv[1]], color, layer: tile_texture.index },
                ModelVertex { position: [x + 1.0, y, 0.0], uv: [tile_texture.uv[0], tile_texture.uv[1]], color, layer: tile_texture.index },
                ModelVertex { position: [x + 1.0, y + 1.0, 0.0], uv: [tile_texture.uv[0], 0.0], color, layer: tile_texture.index },
                ModelVertex { position: [x, y + 1.0, 0.0], uv: [0.0, 0.0], color, layer: tile_texture.index },
            ];
            let indices: Vec<u32> = QUAD_INDICES.iter().map(|x| x + (i * CHUNK_DIM + j) as u32 * 4).collect();
            model_vertices.extend(vertices);
            model_indices.extend(indices);
        }
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

// pub fn construct_height_map_mesh(handle: &mut RainHandle, chunk: &ChunkInfo) -> ModelMesh {
//     let mut model_vertices: Vec<ModelVertex> = Vec::new();
//     let mut model_indices: Vec<u32> = Vec::new();

//     for (i, row) in chunk.data.iter().enumerate() {
//         for (j, (height, biome)) in row.iter().enumerate() {
//             let mut color_data = glam::Vec3::new(1.0, 1.0, 1.0);
//             color_data = color_data * (*height / MAX_HEIGHT) as f32;
//             let color = match biome {
//                 BiomeType::Forest => Color::GREEN,
//                 BiomeType::Ocean => Color::BLUE,
//                 BiomeType::Coast => Color::YELLOW,
//                 BiomeType::River => Color::CYAN,
//                 BiomeType::None => Color::from_f32(color_data.x, color_data.y, color_data.z, 1.0),
//             };
//             // let color = if *height > 1.0 {
//             //     Color::from_f32(color_data.x, color_data.y, color_data.z, 1.0)
//             // } else {
//             //     Color::BLUE
//             // };
    
//             let color = Color::rain_color_to_array(&color);
    
//             let texture = handle.fetch_texture("").unwrap();
//             let x = (chunk.position.x * CHUNK_DIM as i32) as f32 + i as f32;
//             let y = (chunk.position.y * CHUNK_DIM as i32) as f32 + j as f32;
            
//             let vertices = vec![
//                 ModelVertex { position: [x, y, 0.0], uv: [0.0, texture.uv[1]], color, layer: texture.index },
//                 ModelVertex { position: [x + 1.0, y, 0.0], uv: [texture.uv[0], texture.uv[1]], color, layer: texture.index },
//                 ModelVertex { position: [x + 1.0, y + 1.0, 0.0], uv: [texture.uv[0], 0.0], color, layer: texture.index },
//                 ModelVertex { position: [x, y + 1.0, 0.0], uv: [0.0, 0.0], color, layer: texture.index },
//             ];
//             let indices: Vec<u32> = QUAD_INDICES.iter().map(|x| x + (i * CHUNK_DIM as usize + j) as u32 * 4).collect();
//             model_vertices.extend(vertices);
//             model_indices.extend(indices);
//         }
//     }

    
//     ModelMesh {
//         vertices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("chunk_vertex_buffer"),
//             contents: bytemuck::cast_slice(&model_vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         }),
//         indices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("chunk_index_buffer"),
//             contents: bytemuck::cast_slice(&model_indices),
//             usage: wgpu::BufferUsages::INDEX,
//         }),
//         num_indices: model_indices.len() as u32,
//         array_id: ARRAY_256X256_ID,
//     }
// }

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