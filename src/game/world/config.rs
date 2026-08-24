use serde::Deserialize;

use crate::game::world::object::ObjectType;

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
    pub tile_rule: Vec<((f64, f64), String)>,
    pub object_rule: Vec<((f64, f64), f64, Vec<String>, ObjectType)>,
    pub default_tile: String,
}