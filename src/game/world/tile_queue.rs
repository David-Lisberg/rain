use std::collections::VecDeque;

use hecs::Entity;
use rain::engine::core::RainHandle;

use crate::{State, game::world::{chunk::{BLOB_TILESET, CHUNK_DIM, ChunkPosition}, tile::{Tile, TilePosition}, tileset::ChunkTileSet}};

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

            let tile_type = chunk.tiles[1][tile_position.x][tile_position.y].type_id;
            if tile_type == state.tile_registry.get_id("none").unwrap() {
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
            let tile_data = state.tile_registry.from_id(tile_type).unwrap();
            if tile_data.tileset.is_some() {
                let old_mask = tileset[1][tile_position.x][tile_position.y].unwrap_or(0);
                let mut mask: u8 = 0;
                for ((x_offset, y_offset), weight) in BLOB_TILESET {
                    let (adjacent_chunk_position, adjacent_tile_position) = get_adjacent_chunk_position_tile_position(
                        tile_position, chunk_position, x_offset, y_offset
                    );
                    let adjacent_tile_type = match state.chunks.get(&adjacent_chunk_position) {
                        Some(chunk) => chunk.tiles[1][adjacent_tile_position.x][adjacent_tile_position.y].type_id,
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

const CONNECTOR_ADJACENT: [((i32, i32), &'static str); 4] = [((0, 1), "north"), ((1, 0), "east"), ((0, -1), "south"), ((-1, 0), "west")];

pub fn system_update_tile_connector(state: &mut State) {
    let mut tiles_to_update: Vec<(ChunkPosition, TilePosition, Tile)> = Vec::new();
    let mut updated: Vec<(ChunkPosition, TilePosition)> = Vec::new();

    if !state.tile_connector_queue.queue.is_empty() {
        while let Some((chunk_position, tile_position)) = state.tile_connector_queue.pop() {
            updated.push((chunk_position, tile_position));
            let Some(chunk) = state.chunks.get_mut(&chunk_position) else {
                continue;
            };

            let mut tile = chunk.tiles[1][tile_position.x][tile_position.y];
            let tile_data = state.tile_registry.from_id(tile.type_id).unwrap();
            if let Some(connector) = &tile_data.connector {
                let previous_state = tile.state;
                for ((x_offset, y_offset), property) in CONNECTOR_ADJACENT {
                    let (adjacent_chunk_position, adjacent_tile_position) = get_adjacent_chunk_position_tile_position(
                        tile_position, chunk_position, x_offset, y_offset
                    );
                    let Some(adjacent_chunk) = state.chunks.get(&adjacent_chunk_position) else {
                        continue;
                    };
                    let adjacent_tile_type = adjacent_chunk.tiles[1][adjacent_tile_position.x][adjacent_tile_position.y].type_id;

                    let can_connect = connector.can_connect.iter().any(|tile_name| {
                        state.tile_registry.get_id(tile_name).unwrap() == adjacent_tile_type
                    });

                    if can_connect {
                        tile.set_property(tile_data, &state.tile_property_registry, property, "true");
                    } else {
                        tile.set_property(tile_data, &state.tile_property_registry, property, "false");
                    }

                    let adjacent_tile_data = state.tile_registry.from_id(adjacent_tile_type).unwrap();
                    if adjacent_tile_data.connector.is_some() && !updated.contains(&(adjacent_chunk_position, adjacent_tile_position)) {
                        state.tile_connector_queue.push(adjacent_chunk_position, adjacent_tile_position);
                    }
                }
                if previous_state != tile.state {
                    tiles_to_update.push((chunk_position, tile_position, tile));
                }
            } else {
                for ((x_offset, y_offset), _) in CONNECTOR_ADJACENT {
                    let (adjacent_chunk_position, adjacent_tile_position) = get_adjacent_chunk_position_tile_position(
                        tile_position, chunk_position, x_offset, y_offset
                    );
                    let Some(adjacent_chunk) = state.chunks.get(&adjacent_chunk_position) else {
                        continue;
                    };
                    let adjacent_tile_type = adjacent_chunk.tiles[1][adjacent_tile_position.x][adjacent_tile_position.y].type_id;

                    let adjacent_tile_data = state.tile_registry.from_id(adjacent_tile_type).unwrap();
                    if adjacent_tile_data.connector.is_some() {
                        state.tile_connector_queue.push(adjacent_chunk_position, adjacent_tile_position);
                    }

                }
            }
            state.chunks_to_reload.insert(chunk_position);
        }
    }

    for (chunk_position, tile_position, tile) in tiles_to_update {
        let Some(chunk) = state.chunks.get_mut(&chunk_position) else {
            continue;
        };
        chunk.tiles[1][tile_position.x][tile_position.y] = tile;
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