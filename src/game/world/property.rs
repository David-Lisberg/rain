use std::collections::HashMap;

use serde::Deserialize;

use crate::game::world::tile::{Tile, TileData};

pub type TilePropertyRegistry = HashMap<String, TilePropertyData>;

#[derive(Deserialize)]
pub struct TilePropertyDataRaw {
    pub name: String,
    values: Vec<String>,
}

pub struct TilePropertyData {
    pub name: String,
    pub values: Vec<String>,
    pub value_map: HashMap<String, u32>,
    pub shift: u32,
}

impl TilePropertyData {
    pub fn new(name: String, values: Vec<String>) -> Self {
        let mut value_map: HashMap<String, u32> = HashMap::new();
        for (i, value) in values.iter().enumerate() {
            value_map.insert(value.clone(), i as u32);
        }
        let shift = usize::BITS - (values.len() - 1).leading_zeros();

        Self {
            name,
            values,
            value_map,
            shift,
        }
    }

    pub fn from_raw(raw: TilePropertyDataRaw) -> Self {
        Self::new(raw.name, raw.values)
    }
}

impl Tile {
    pub fn set_property(&mut self, tile_data: &TileData, property_registry: &TilePropertyRegistry, property: &str, value: &str) {
        let Some(offset) = tile_data.property_map.get(property) else {
            return;
        };
        let property_data = property_registry.get(property).unwrap();
        let real_value = property_data.value_map.get(value).unwrap();
        let state = *real_value << *offset;
        self.state |= state;
    }

    pub fn get_property(&mut self, tile_data: &TileData, property_registry: &TilePropertyRegistry, property: &str) -> Option<String> {
        let Some(offset) = tile_data.property_map.get(property) else {
            return None;
        };
        let property_data = property_registry.get(property).unwrap();
        let mut index = self.state >> offset;
        index &= (1 << property_data.shift) - 1;
        Some(property_data.values[index as usize].clone())
    }
}