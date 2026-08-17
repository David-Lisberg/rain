use glam::Vec2;
use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::resource::ResourceManager;
use rain::engine::texture::Texture;
use serde::{Deserialize, Serialize};
use rain::engine::color::Color;
use rain::engine::component::*;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Arc;

use crate::game::player::action::PLAYER_REACH;
use crate::game::player::movement::Player;
use crate::game::world::chunk::{CHUNK_DIM, position_to_chunk_position};

pub type TileRegistry = HashMap<TileType, TileData>;

#[derive(Deserialize)]
pub struct TileData {
    pub tile_type: TileType,
    pub texture: String,
}

pub struct Tile {
    pub _type: TileType, 
}

pub struct TilePosition(pub usize);

pub fn position_to_tile_position(x: f32, y: f32) -> TilePosition {
    TilePosition((y as usize % CHUNK_DIM) * CHUNK_DIM + x as usize % CHUNK_DIM)
}

#[repr(u8)]
#[derive(PartialEq, Clone, Deserialize, PartialOrd, Copy, Ord, Eq, Debug, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    Water = 0,
    Clay = 1,
    Sand = 2,
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

pub struct TileHighlight;

impl TileType {
    pub fn fetch_tileset(&self, resource_manager: &ResourceManager) -> Option<Arc<Texture>> {
        match self {
            TileType::Grass => Some(resource_manager.fetch_texture("tile_grass_tileset").unwrap()),
            TileType::Grass2 => Some(resource_manager.fetch_texture("tile_grass2_tileset").unwrap()),
            TileType::Dirt => Some(resource_manager.fetch_texture("tile_dirt_tileset").unwrap()),
            TileType::Stone => Some(resource_manager.fetch_texture("tile_stone_tileset").unwrap()),
            TileType::Sand => Some(resource_manager.fetch_texture("tile_sand_tileset").unwrap()),
            TileType::Mud => Some(resource_manager.fetch_texture("tile_mud_tileset").unwrap()),
            TileType::Clay => Some(resource_manager.fetch_texture("tile_clay_tileset").unwrap()),
            _ => None,
        }
    }

    pub fn has_tileset(&self) -> bool {
        match self {
            TileType::Grass => true,
            TileType::Grass2 => true,
            TileType::Dirt => true,
            TileType::Stone => true,
            TileType::Sand => true,
            TileType::Mud => true,
            TileType::Clay => true,
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

pub fn system_tile_highlight(handle: &mut RainHandle) {
    let mouse_position = handle.screen_position_to_world_position(handle.mouse_position());
    let player_position = handle.world.query::<(&Player, &Position2D)>().iter().next().unwrap().1.1.0;
    let mut to_remove_visible: Option<Entity> = None;
    let mut to_add_visible: Option<Entity> = None;

    for (e, (_, position, visible)) in handle.world.query_mut::<(&TileHighlight, &mut Position2D, Option<&Visible>)>() {
        let distance = (mouse_position - player_position).length();
        if distance > PLAYER_REACH && visible.is_some() {
            to_remove_visible = Some(e);
        } else if distance <= PLAYER_REACH && visible.is_none() {
            to_add_visible = Some(e);
        }
        let chunk_position = position_to_chunk_position(mouse_position.x, mouse_position.y);
        let tile_position = Vec2::new((chunk_position.x + mouse_position.x as i32 % CHUNK_DIM as i32) as f32, 
            (chunk_position.y + mouse_position.y as i32 % CHUNK_DIM as i32) as f32) + 0.5;
        position.0 = tile_position;
    }

    if let Some(e) = to_remove_visible {
        handle.world.remove_one::<Visible>(e).unwrap();
    }
    if let Some(e) = to_add_visible {
        handle.world.insert_one(e, Visible).unwrap();
    }
}