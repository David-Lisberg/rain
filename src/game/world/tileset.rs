use std::collections::VecDeque;

use hecs::Entity;
use rain::engine::core::RainHandle;

use crate::State;
use crate::game::world::chunk::{BLOB_TILESET, CHUNK_DIM, ChunkPosition};
use crate::game::world::tile::{TilePosition, TileType};

pub struct TileQueue {
    queue: VecDeque<(ChunkPosition, TilePosition)>,
    current_chunk: ChunkPosition,
}

impl TileQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current_chunk: ChunkPosition::new(0, 0),
        }
    }

    pub fn pop(&mut self) -> Option<(ChunkPosition, TilePosition)> {
        if let Some(index) = self.queue.iter().position(|x| x.0 == self.current_chunk) {
            return self.queue.remove(index);
        }
        if let Some(tile) = self.queue.pop_front() {
            self.current_chunk = tile.0;
            Some(tile)
        } else {
            None
        }
    }

    pub fn push(&mut self, chunk_position: ChunkPosition, tile_position: TilePosition) {
        if !self.queue.contains(&(chunk_position, tile_position)) {
            self.queue.push_back((chunk_position, tile_position));
            self.current_chunk = chunk_position;
        }
    }
}

pub fn system_update_tiles(handle: &mut RainHandle, state: &mut State) {
    if !state.tile_queue.queue.is_empty() {
        let mut chunk_entity: Option<Entity>; 
        while let Some((chunk_position, tile_position)) = state.tile_queue.pop() {
            if let Some((entity, _)) = handle.world.query::<&ChunkPosition>().iter().find(|x| *x.1 == chunk_position) {
                chunk_entity = Some(entity);
            } else {
                continue;
            }
            
            let Ok(mut tileset) = handle.world.remove_one::<[ChunkTileSet; 2]>(chunk_entity.unwrap()) else {
                continue;
            };
            let Some(chunk) = state.chunks.get(&chunk_position) else {
                continue;
            };

            let tile_type = chunk.tiles[1][tile_position.x][tile_position.y]._type;
            if tile_type == TileType::None {
                let mask = tileset[1][tile_position.x][tile_position.y].unwrap_or(0);
                for ((x_offset, y_offset), weight) in BLOB_TILESET {
                    if mask & weight != 0 {
                        let (adjacent_chunk_position, adjacent_tile_position) = get_adjacent_chunk_position_tile_position(
                            tile_position, chunk_position, x_offset, y_offset
                        );
                        state.tile_queue.push(adjacent_chunk_position, adjacent_tile_position);
                    }
                }
                continue;
            }
            let tile_data = state.tile_registry.get(&tile_type).unwrap();
            if tile_data.tileset.is_some() {
                let old_mask = tileset[1][tile_position.x][tile_position.y].unwrap_or(0);
                let mut mask: u8 = 0;
                for ((x_offset, y_offset), weight) in BLOB_TILESET {
                    let (adjacent_chunk_position, adjacent_tile_position) = get_adjacent_chunk_position_tile_position(
                        tile_position, chunk_position, x_offset, y_offset
                    );
                    let adjacent_tile_type = match state.chunks.get(&adjacent_chunk_position) {
                        Some(chunk) => chunk.tiles[1][adjacent_tile_position.x][adjacent_tile_position.y]._type,
                        None => continue,
                    };
                    if adjacent_tile_type == tile_type {
                        mask |= weight;
                    }
                    if (old_mask ^ mask) & weight != 0 {
                        state.tile_queue.push(adjacent_chunk_position, adjacent_tile_position);
                    }
                }
                tileset[1][tile_position.x][tile_position.y] = Some(mask);
            }
            state.chunks_to_reload.insert(chunk_position);
            handle.world.insert_one(chunk_entity.unwrap(), tileset).unwrap();
        }
    }
}

fn get_adjacent_chunk_position_tile_position(tile_position: TilePosition, chunk_position: ChunkPosition, x_offset: i32, y_offset: i32) -> (ChunkPosition, TilePosition) {
    let adjacent = (tile_position.x as i32 + x_offset, tile_position.y as i32 + y_offset);
    let chunk_offset_x = if adjacent.0 < 0 {
        -1
    } else if adjacent.0 >= CHUNK_DIM as i32 {
        1
    } else {
        0
    };
    let chunk_offset_y = if adjacent.1 < 0 {
        -1
    } else if adjacent.1 >= CHUNK_DIM as i32 {
        1
    } else {
        0
    };
    let adjacent_chunk_position = ChunkPosition::new(chunk_position.x + chunk_offset_x, chunk_position.y + chunk_offset_y);
    let adjacent_x = adjacent.0.rem_euclid(CHUNK_DIM as i32) as usize;
    let adjacent_y = adjacent.1.rem_euclid(CHUNK_DIM as i32) as usize;
    (adjacent_chunk_position, TilePosition { x: adjacent_x, y: adjacent_y })
}

// Used for tile masks
//
// pub struct ChunkTileSet(pub [[Vec<(TileType, u8)>; CHUNK_DIM]; CHUNK_DIM]);

pub type ChunkTileSet = [[Option<u8>; CHUNK_DIM]; CHUNK_DIM];

const MASK_TO_TILE: [(u8, u8); 47] = [
    (0, 0),
    (1, 1),
    (4, 2),
    (16, 3),
    (64, 4),
    (5, 5),
    (20, 6),
    (80, 7),
    (65, 8),
    (7, 9),
    (28, 10),
    (112, 11),
    (193, 12),
    (17, 13),
    (68, 14),
    (21, 15),
    (84, 16),
    (81, 17),
    (69, 18),
    (23, 19),
    (92, 20),
    (113, 21),
    (197, 22),
    (29, 23),
    (116, 24),
    (209, 25),
    (71, 26),
    (31, 27),
    (124, 28),
    (241, 29),
    (199, 30),
    (85, 31),
    (87, 32),
    (93, 33),
    (117, 34),
    (213, 35),
    (95, 36),
    (125, 37),
    (245, 38),
    (215, 39),
    (119, 40),
    (221, 41),
    (127, 42),
    (253, 43),
    (247, 44),
    (223, 45),
    (255, 46)
];

const TILEMAP_WIDTH: f32 = 11.0;
const TILEMAP_HEIGHT: f32 = 5.0;
pub const UV_LOOKUP: [[f32; 4]; 47] = [
    [3.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [3.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 4.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [4.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 5.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [7.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 8.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [0.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 1.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [2.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 3.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [10.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 11.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [10.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT, 11.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 4.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 5.0 / TILEMAP_HEIGHT],
    [8.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 9.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [9.0 / TILEMAP_WIDTH, 0.0 / TILEMAP_HEIGHT, 10.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 3.0 / TILEMAP_HEIGHT],
    [5.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 6.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [6.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 7.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
    [1.0 / TILEMAP_WIDTH, 1.0 / TILEMAP_HEIGHT, 2.0 / TILEMAP_WIDTH, 2.0 / TILEMAP_HEIGHT],
];

pub fn generate_tileset_lookup() -> [u8; 256] {
    let mut tile_lookup: [u8; 256] = [0; 256];

    for i in 0..=255 {
        let effective_mask = get_effective_mask(i);
        for (mask, tile) in MASK_TO_TILE {
            if effective_mask == mask {
                tile_lookup[i as usize] = tile;
            }
        }
    }

    tile_lookup
}

fn get_effective_mask(mask: u8) -> u8 {
    let n  = mask & 1 != 0;
    let ne = mask & 2 != 0;
    let e  = mask & 4 != 0;
    let se = mask & 8 != 0;
    let s  = mask & 16 != 0;
    let sw = mask & 32 != 0;
    let w  = mask & 64 != 0;
    let nw = mask & 128 != 0;

    let effective = (n  as u8)
        | ((n && e && ne) as u8) << 1
        | (e  as u8) << 2
        | ((e && s && se) as u8) << 3
        | (s  as u8) << 4
        | ((s && w && sw) as u8) << 5
        | (w  as u8) << 6
        | ((n && w && nw) as u8) << 7;

    effective
}