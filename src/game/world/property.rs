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
    pub fn set_property(&mut self, tile_data: &TileData, property_registry: &TilePropertyRegistry, property_name: &str, value: &str) {
        let Some(property_index) = tile_data.property_map.get(property_name) else {
            return;
        };
        let property = &tile_data.properties[*property_index];
        let property_data = property_registry.get(&property.property_type).unwrap();
        let real_value = property_data.value_map.get(value).unwrap();
        let state = *real_value << property.offset;
        self.state |= state;
    }

    /* this doesn't work */
    pub fn set_property_mask(&mut self, tile_data: &TileData, property: &str, value: u32) {
        let Some(property_index) = tile_data.property_map.get(property) else {
            return;
        };
        let state = value << tile_data.properties[*property_index].offset;
        self.state |= state;
    }

    pub fn get_property(&self, tile_data: &TileData, property_registry: &TilePropertyRegistry, property_name: &str) -> Option<String> {
        let Some(property_index) = tile_data.property_map.get(property_name) else {
            return None;
        };
        let property = &tile_data.properties[*property_index];
        let property_data = property_registry.get(&property.property_type).unwrap();
        let mut index = self.state >> property.offset;
        index &= (1 << property_data.shift) - 1;
        Some(property_data.values[index as usize].clone())
    }
    
    pub fn get_property_mask(&self, tile_data: &TileData, property_registry: &TilePropertyRegistry, property: &str) -> Option<u32> {
        let Some(property_index) = tile_data.property_map.get(property) else {
            return None;
        };
        let property_data = property_registry.get(property).unwrap();
        let index = self.state >> tile_data.properties[*property_index].offset;
        Some(index & (1 << property_data.shift) - 1)
    }
}