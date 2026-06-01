use serde::Deserialize;

use crate::game::world::object::ObjectType;
use crate::game::world::tile::TileType;

pub type WorldGenConfig = Vec<BiomeConfig>;

#[derive(PartialEq, Deserialize)]
pub enum BiomeType {
    Forest,
    River,
    Ocean,
    Coast,
    None
}

#[derive(Deserialize)]
pub struct BiomeConfig {
    #[serde(rename = "type")]
    pub _type: BiomeType,
    pub tile_rule: Vec<((f64, f64), TileType)>,
    pub object_rule: Vec<((f64, f64), f64, Vec<TileType>, ObjectType)>,
    pub default_tile: TileType,
}