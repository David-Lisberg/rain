use glam::Vec2;
use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::texture::Texture;
use serde::{Deserialize, Serialize};
use rain::engine::color::Color;
use rain::engine::component::*;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Arc;

use crate::game::player::action::PLAYER_REACH;
use crate::game::player::item::{Item, ToolType};
use crate::game::player::movement::Player;
use crate::game::world::chunk::CHUNK_DIM;
use crate::game::world::property::TilePropertyRegistry;

pub struct TileRegistry {
    data: Vec<TileData>,
    ids: HashMap<String, u32>
}

impl TileRegistry {
    pub fn new(data: Vec<TileData>) -> Self {
        let mut ids: HashMap<String, u32> = HashMap::new();
        for (i, tile_data) in data.iter().enumerate() {
            ids.insert(tile_data.name.clone(), i as u32);
        }

        Self {
            data,
            ids,
        }
    }

    pub fn from_name(&self, name: &str) -> Option<&TileData> {
        let Some(id) = self.ids.get(name) else {
            return None;
        };
        self.data.get(*id as usize)
    }

    pub fn from_id(&self, id: u32) -> Option<&TileData> {
        self.data.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.ids.get(name).cloned()
    }
}

#[derive(Deserialize)]
pub struct TileDataRaw {
    pub name: String,
    pub texture: String,
    pub collidable: bool,
    pub swimmable: bool,
    pub tileset: Option<String>,
    pub break_level: Option<i32>,
    pub required_tool: Option<ToolType>,
    pub drops: Option<Vec<(Item, i32)>>,
    pub properties: Option<Vec<TileProperty>>,
}

pub struct TileData {
    pub name: String,
    pub texture: Arc<Texture>,
    pub collidable: bool,
    pub swimmable: bool,
    pub tileset: Option<String>,
    pub break_level: Option<i32>,
    pub required_tool: ToolType,
    pub drops: Vec<(Item, i32)>,
    pub properties: Vec<TileProperty>,
    pub property_map: HashMap<String, u32>,
    pub default_state: u32,
}

impl TileData {
    pub fn from_raw(handle: &mut RainHandle, property_registry: &TilePropertyRegistry, raw: TileDataRaw) -> Self {
        let properties = raw.properties.unwrap_or(Vec::new());
        let mut property_map: HashMap<String, u32> = HashMap::new();
        let mut i = 0;
        let mut default_state: u32 = 0;
        for property in properties.iter() {
            let property_data = property_registry.get(&property.property_type).unwrap();
            property_map.insert(property.name.clone(), i);
            if let Some(default) = &property.default {
                let value = property_data.value_map.get(default).unwrap();
                default_state |= *value << i;
            }
            i += property_data.shift;

        }

        Self {
            name: raw.name,
            texture: handle.fetch_texture(&raw.texture).unwrap(),
            collidable: raw.collidable,
            swimmable: raw.swimmable,
            tileset: raw.tileset,
            break_level: raw.break_level,
            required_tool: raw.required_tool.unwrap_or(ToolType::None),
            drops: raw.drops.unwrap_or(Vec::new()),
            properties,
            property_map,
            default_state,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct TileProperty {
    name: String,
    property_type: String,
    default: Option<String>,
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub type_id: u32,
    pub state: u32,
}

impl Tile {
    pub fn new(type_id: u32) -> Self {
        Self {
            type_id,
            state: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

impl TilePosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
        }
    }
}

pub fn position_to_tile_position(x: f32, y: f32) -> TilePosition {
    TilePosition {
        x: (x.floor() as i32).rem_euclid(CHUNK_DIM as i32) as usize,
        y: (y.floor() as i32).rem_euclid(CHUNK_DIM as i32) as usize,
    }
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