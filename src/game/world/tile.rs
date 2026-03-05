use glam::Vec2;
use rain::engine::core::RainHandle;
use rain::engine::texture::Texture;
use serde::{Deserialize, Serialize};
use rain::engine::color::Color;
use rain::engine::component::*;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Arc;

pub struct Tile {
    pub _type: TileType, 
}

pub enum TileType {
    Grass,
    Dirt,
    Stone,
    Cobblestone,
    Water,
    Sand,
}

#[derive(Serialize, Deserialize)]
pub struct Vec2JSON {
    pub x: f32,
    pub y: f32,
}
#[derive(Serialize, Deserialize)]
pub struct TileJSON {
    pub position: Vec2JSON,
    pub size: Vec2JSON,
    pub color: Color,
}

impl TileType {
    pub fn fetch_texture(&self, handle: &mut RainHandle) -> Arc<Texture> {
        match self {
            TileType::Dirt => handle.fetch_texture("tile_dirt").unwrap(),
            TileType::Grass => handle.fetch_texture("tile_grass").unwrap(),
            TileType::Stone => handle.fetch_texture("tile_stone").unwrap(),
            TileType::Cobblestone => handle.fetch_texture("tile_cobblestone").unwrap(),
            TileType::Water => handle.fetch_texture("tile_water").unwrap(),
            TileType::Sand => handle.fetch_texture("tile_sand").unwrap(),
        }
    }
}

pub fn write_tile() {
    let tile = TileJSON {
        position: Vec2JSON { x: 6.0, y: 0.0 },
        size: Vec2JSON { x: 1.0, y: 1.0 },
        color: Color::BLUE
    };
    let serialized = serde_json::to_string(&tile).unwrap();
    let path = String::from("res/saves/test.txt");
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Error opening file.");
    writeln!(&mut file, "{}", serialized).expect("Error writing tile.");
}

pub fn read_tile(handle: &mut RainHandle, file: &mut File) {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Error: Failed to read file.");
    let tile_json: TileJSON = serde_json::from_str(&buffer).expect("Error: Failed to parse tile json.");
    handle.world.spawn((Sprite, Visible, Position2D{ x: tile_json.position.x, y: tile_json.position.y },
        Scale2D(Vec2::new(tile_json.size.x, tile_json.size.y)), tile_json.color));
}