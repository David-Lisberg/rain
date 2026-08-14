use std::collections::{HashMap, VecDeque};
use std::ops::Range;

use glam::{IVec2, Vec2};
use hecs::Entity;
use rain::engine::color::Color;
use rain::engine::component::*;
use rain::engine::core::RainHandle;
use rain::engine::mesh::ModelMesh;
use rain::engine::resource::ARRAY_512X512_ID;
use rain::engine::vertex::{ModelVertex, QUAD_INDICES};
use rand::RngExt;
use wgpu::util::DeviceExt;

use crate::State;
use crate::game::core::collision::Collider;
use crate::game::utility::noise::{noise_normalize, octave_noise_2d};
use crate::game::world::config::BiomeType;
use crate::game::world::generation::CHUNK_GENERATION_DISTANCE;
use crate::game::world::tile::{Tile, TileType};
use crate::game::world::object::{Object, ObjectType, reload_object_mesh};
use crate::game::player::movement::Player;
use crate::game::world::tileset::{ChunkTileSet, UV_LOOKUP};

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
    data: [[(f64, BiomeType); CHUNK_DIM]; CHUNK_DIM],
}

pub const CHUNK_DIM: usize = 32; /* indices should be u32s if CHUNK_DIM > 64 */
const NOISE_OBJECT_SCALE_FACTOR: f64 = 0.037;
const NOISE_DENSITY_SCALE_FACTOR: f64 = 0.0243;
const NOISE_WATER_BANK_SCALE_FACTOR: f64 = 0.035;
const ADJACENT_TILE: [IVec2; 4] = [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)];

const SCALE_CONTINENTALNESS: f64 = 0.007346;
const SCALE_EROSION: f64 = 0.02013;
const SCALE_RIVER: f64 = 0.0049;

const RIVER_THRESHOLD: f64 = 0.11;

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
    ChunkInfo { data }
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

pub fn generate_chunk(handle: &mut RainHandle, state: &mut State, chunk_position: ChunkPosition) -> ChunkData {
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

            let mut object: Option<(ObjectType, Vec2)> = None;

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
                                object = Some((*object_type, position));
                                break;
                            }
                        }
                    }
                    if object.is_some() {
                        break;
                    }
                }
            }
            if let Some((object_type, position)) = object {
                let object_data = state.object_registry.get(&object_type).unwrap();
                let object = Object::from_data(handle, object_data, position);

                objects.push(object);
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

const TRANSITION_LAYER_DEPTH: f32 = 0.0001;

pub fn construct_chunk_mesh(handle: &mut RainHandle, chunk: &ChunkData, tileset: &ChunkTileSet, tileset_lookup: &[u8; 256]) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u32> = Vec::new();

    let color = Color::rain_color_to_array(&Color::WHITE);
    let mut num_indices = 0;

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
            let indices: Vec<u32> = QUAD_INDICES.iter().map(|x| x + num_indices).collect();
            model_vertices.extend(vertices);
            model_indices.extend(indices);
            num_indices += 4;

            // let transitions = &tileset.0[i][j];
            // if !transitions.is_empty() {
            //     for (k, transition) in transitions.iter().enumerate() {
            //         let transition_texture = transition.0.fetch_tileset(&handle.resource_manager).unwrap();
            //         let tile_index = tileset_lookup[transition.1 as usize];
            //         let mut uv_rect = UV_LOOKUP[tile_index as usize];
            //         uv_rect[0] *= transition_texture.uv[0];
            //         uv_rect[1] *= transition_texture.uv[1];
            //         uv_rect[2] *= transition_texture.uv[0];
            //         uv_rect[3] *= transition_texture.uv[1];
                    
            //         let depth = TRANSITION_LAYER_DEPTH * (k as f32 + 1.0);
            //         let vertices = vec![
            //             ModelVertex { position: [x, y, depth], uv: [uv_rect[0], uv_rect[3]], color, layer: transition_texture.index },
            //             ModelVertex { position: [x + 1.0, y, depth], uv: [uv_rect[2], uv_rect[3]], color, layer: transition_texture.index },
            //             ModelVertex { position: [x + 1.0, y + 1.0, depth], uv: [uv_rect[2], uv_rect[1]], color, layer: transition_texture.index },
            //             ModelVertex { position: [x, y + 1.0, depth], uv: [uv_rect[0], uv_rect[1]], color, layer: transition_texture.index },
            //         ];

            //         let indices: Vec<u32> = QUAD_INDICES.iter().map(|x| x + num_indices).collect();
            //         model_vertices.extend(vertices);
            //         model_indices.extend(indices);
            //         num_indices += 4;
            //     }
            // }
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
        array_id: ARRAY_512X512_ID,
    }
}

pub fn system_manage_chunks(handle: &mut RainHandle, state: &mut State) {
    let mut to_deload: Vec<Entity> = Vec::new();
    let mut to_load: Vec<(Entity, ChunkPosition)> = Vec::new();
    let mut updated = false;
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
                    to_load.push((e, *chunk));
                }
            }
        }
    }

    if !to_deload.is_empty() || !to_load.is_empty() {
        reload_object_mesh(handle, state);
    }

    for e in to_deload {
        updated = true;
        handle.world.remove_one::<ModelMesh>(e).unwrap();
    }
    for (e, chunk_position) in to_load {
        updated = true;
        let tileset = if let Ok(t) = handle.world.remove_one::<ChunkTileSet>(e) {
            t
        } else {
            let mut padded: [[TileType; CHUNK_DIM + 2]; CHUNK_DIM + 2] = [[TileType::Water; CHUNK_DIM + 2]; CHUNK_DIM + 2];
            if let Some(chunk) = state.chunks.get(&chunk_position) {
                for x in 0..CHUNK_DIM {
                    for y in 0..CHUNK_DIM {
                        padded[x + 1][y + 1] = chunk.tiles[x][y]._type;
                    }
                }
            }
            for ((x, y), (range_x, range_y), (padded_x, padded_y)) in ADJACENT_BORDER {
                let position = ChunkPosition::new(chunk_position.x + x, chunk_position.y + y);
                if !state.chunks.contains_key(&position) {
                    let chunk = generate_chunk(handle, state, position);
                    handle.world.spawn((position,));
                    state.chunks.insert(position, chunk);
                }
                let chunk = state.chunks.get(&position).unwrap();
                for (i, padded_i) in range_x.zip(padded_x) {
                    for (j, padded_j) in range_y.clone().zip(padded_y.clone()) {
                        padded[padded_i][padded_j] = chunk.tiles[i][j]._type;
                    }
                }
            }
            construct_chunk_tileset(padded)
        };
        if let Some(chunk) = state.chunks.get(&chunk_position) {
            let mesh = construct_chunk_mesh(handle, chunk, &tileset, &state.tileset_lookup);
            handle.world.insert(e, (mesh, Visible, tileset)).unwrap();
        }
    }
    if updated {
        reload_object_mesh(handle, state);
    }
}

const ADJACENT_BORDER: [((i32, i32), (Range<usize>, Range<usize>), (Range<usize>, Range<usize>)); 8] = [
    ((0, 1), (0..CHUNK_DIM, 0..1), (1..(CHUNK_DIM + 1), (CHUNK_DIM + 1)..(CHUNK_DIM + 2))),
    ((1, 1), (0..1, 0..1), ((CHUNK_DIM + 1)..(CHUNK_DIM + 2), (CHUNK_DIM + 1)..(CHUNK_DIM + 2))),
    ((1, 0), (0..1, 0..CHUNK_DIM), ((CHUNK_DIM + 1)..(CHUNK_DIM + 2), 1..(CHUNK_DIM + 1))),
    ((1, -1), (0..1, (CHUNK_DIM - 1)..CHUNK_DIM), ((CHUNK_DIM + 1)..(CHUNK_DIM + 2), 0..1)),
    ((0, -1), (0..CHUNK_DIM, (CHUNK_DIM - 1)..CHUNK_DIM), (1..(CHUNK_DIM + 1), 0..1)),
    ((-1, -1), ((CHUNK_DIM - 1)..CHUNK_DIM, (CHUNK_DIM - 1)..CHUNK_DIM), (0..1, 0..1)),
    ((-1, 0), ((CHUNK_DIM - 1)..CHUNK_DIM, 0..CHUNK_DIM), (0..1, 1..(CHUNK_DIM + 1))),
    ((-1, 1), ((CHUNK_DIM - 1)..CHUNK_DIM, 0..1), (0..1, (CHUNK_DIM + 1)..(CHUNK_DIM + 2)))
];
const BLOB_TILESET: [((i32, i32), u8); 8] = [((0, 1), 199), ((1, 1), 2), ((1, 0), 31), ((1, -1), 8), ((0, -1), 124), ((-1, -1), 32), ((-1, 0), 241), ((-1, 1), 128)];

fn construct_chunk_tileset(padded: [[TileType; CHUNK_DIM + 2]; CHUNK_DIM + 2]) -> ChunkTileSet {
    let mut tileset_masks: [[Vec<(TileType, u8)>; CHUNK_DIM]; CHUNK_DIM] = std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()));

    for x in 0..CHUNK_DIM {
        for y in 0..CHUNK_DIM {
            let tile_type = padded[x + 1][y + 1];
            let mut masks: Vec<(TileType, u8)> = Vec::new();
            for ((x_offset, y_offset), weight) in BLOB_TILESET {
                let adjacent = (x as i32 + x_offset, y as i32 + y_offset);
                let adjacent_tile_type = padded[(adjacent.0 + 1) as usize][(adjacent.1 + 1) as usize];
                if adjacent_tile_type.has_tileset() && adjacent_tile_type > tile_type {
                    if let Some(mask) = masks.iter_mut().find(|(x, _)| *x == adjacent_tile_type) {
                        mask.1 |= weight;
                    } else {
                        masks.push((adjacent_tile_type, weight));
                    }
                }
            }
            masks.sort_by_key(|(i, _)| *i);
            tileset_masks[x][y] = masks;
        }
    }
    ChunkTileSet(tileset_masks)
}