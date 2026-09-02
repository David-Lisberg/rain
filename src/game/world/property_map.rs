use std::collections::HashMap;

use rain::engine::animation::UVRect;
use serde::Deserialize;

pub type TilePropertyTextureMapRegistry = HashMap<String, TilePropertyTextureMap>;

pub struct TilePropertyTextureMap {
    pub entries: Vec<TilePropertyTextureMapEntry>,
}

#[derive(Deserialize, Clone)]
pub struct TilePropertyTextureMapEntryRaw {
    properties: Vec<(String, String)>,
    uv: Option<UVRect>,
}

#[derive(Deserialize, Clone)]
pub struct TilePropertyTextureMapEntry {
    pub properties: Vec<(String, String)>,
    pub uv: UVRect,
}

impl TilePropertyTextureMapEntry {
    pub fn from_raw(raw: TilePropertyTextureMapEntryRaw) -> Self {
        Self {
            properties: raw.properties,
            uv: raw.uv.unwrap_or(UVRect::default()),
        }
    }
}