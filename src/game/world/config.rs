use crate::game::world::tile::TileType;

pub struct WorldGenConfig {
    pub tile_noise: &'static [((f64, f64), TileType)],
    pub default_tile: TileType,
}

impl WorldGenConfig {
    pub fn default() -> Self {
        Self {
            tile_noise: &[
                ((0.85, f64::MAX), TileType::Cobblestone),
                ((0.65, 0.85), TileType::Stone),
                ((0.6, 0.65), TileType::Dirt),
                ((0.48, 0.6), TileType::Grass2),
                ((0.35, 0.48), TileType::Grass),
                ((0.3, 0.35), TileType::Sand),
                ((f64::MIN, 0.3), TileType::Water),
            ],
            default_tile: TileType::Dirt
        }
    }
}