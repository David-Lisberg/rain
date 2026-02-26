use glam::Vec2;

use crate::game::world::tile::Tile;

pub struct ChunkData {
    position: Vec2,
    tiles: [Tile; CHUNK_DIM * CHUNK_DIM],
    tile_entities: Vec<Tile>,
}

pub const CHUNK_DIM: usize = 32;

fn generate_chunk() {

}

fn construct_chunk_model() {

}