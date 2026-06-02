use glam::Vec2;
use rain::engine::core::RainHandle;
use rain::engine::resource::ResourceManager;
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

#[repr(u8)]
#[derive(PartialEq, Clone, Deserialize, PartialOrd, Copy, Ord, Eq, Debug)]
pub enum TileType {
    Water = 0,
    Sand = 1,
    Clay = 2,
    Mud = 3,
    Grass = 4,
    Grass2 = 5,
    Dirt = 6,
    Stone = 7,
    Cobblestone = 8,
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
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            TileType::Dirt => resource_manager.fetch_texture("tile_dirt").unwrap(),
            TileType::Grass => resource_manager.fetch_texture("tile_grass").unwrap(),
            TileType::Grass2 => resource_manager.fetch_texture("tile_grass2").unwrap(),
            TileType::Stone => resource_manager.fetch_texture("tile_stone").unwrap(),
            TileType::Cobblestone => resource_manager.fetch_texture("tile_cobblestone").unwrap(),
            TileType::Water => resource_manager.fetch_texture("tile_water").unwrap(),
            TileType::Sand => resource_manager.fetch_texture("tile_sand").unwrap(),
            TileType::Clay => resource_manager.fetch_texture("tile_clay").unwrap(),
            TileType::Mud => resource_manager.fetch_texture("tile_mud").unwrap(),
        }
    }

    pub fn fetch_tileset(&self, resource_manager: &ResourceManager) -> Option<Arc<Texture>> {
        match self {
            TileType::Grass => Some(resource_manager.fetch_texture("tile_grass_tileset").unwrap()),
            _ => None,
        }
    }

    pub fn has_tileset(&self) -> bool {
        match self {
            TileType::Grass => true,
            _ => false,
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
    handle.world.spawn((Sprite, Visible, Position2D(Vec2::new(tile_json.position.x, tile_json.position.y)),
        Scale2D(Vec2::new(tile_json.size.x, tile_json.size.y)), tile_json.color));
}