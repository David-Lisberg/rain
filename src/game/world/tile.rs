use glam::Vec2;
use hecs::Entity;
use rain::engine::core::RainHandle;
use serde::{Deserialize, Serialize};
use rain::engine::color::Color;
use rain::engine::component::*;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::game::player::action::PLAYER_REACH;
use crate::game::player::item::{Item, ToolType};
use crate::game::player::movement::Player;
use crate::game::world::chunk::CHUNK_DIM;

pub type TileRegistry = HashMap<TileType, TileData>;

#[derive(Deserialize)]
pub struct TileData {
    pub tile_type: TileType,
    pub texture: String,
    pub tileset: Option<String>,
    pub break_level: Option<i32>,
    pub required_tool: Option<ToolType>,
    pub drops: Option<Vec<(Item, i32)>>
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub _type: TileType, 
}

pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

pub fn position_to_tile_position(x: f32, y: f32) -> TilePosition {
    TilePosition {
        x: (x.floor() as i32).rem_euclid(CHUNK_DIM as i32) as usize,
        y: (y.floor() as i32).rem_euclid(CHUNK_DIM as i32) as usize,
    }
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
    WoodFloor = 9,
    WoodWall = 10,
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
        let tile_position = Vec2::new(mouse_position.x.floor() + 0.5, mouse_position.y.floor() + 0.5);
        position.0 = tile_position;
    }

    if let Some(e) = to_remove_visible {
        handle.world.remove_one::<Visible>(e).unwrap();
    }
    if let Some(e) = to_add_visible {
        handle.world.insert_one(e, Visible).unwrap();
    }
}