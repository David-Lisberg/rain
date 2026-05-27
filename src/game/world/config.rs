use crate::game::world::object::ObjectType;
use crate::game::world::tile::TileType;

pub struct WorldGenConfig {
    pub tile_rule: &'static [((f64, f64), TileType)],
    pub object_rule: &'static [((f64, f64), f64, &'static [TileType], ObjectType)],
    pub default_tile: TileType,
}

impl WorldGenConfig {
    pub fn default() -> Self {
        Self {
            tile_rule: &[
                ((0.85, f64::MAX), TileType::Cobblestone),
                ((0.65, 0.85), TileType::Stone),
                ((0.6, 0.65), TileType::Dirt),
                ((0.48, 0.6), TileType::Grass2),
                ((0.35, 0.48), TileType::Grass),
                ((0.3, 0.35), TileType::Sand),
                ((f64::MIN, 0.3), TileType::Water),
            ],
            object_rule : &[
                ((0.45, f64::MAX), 0.05, &[TileType::Grass, TileType::Grass2], ObjectType::Tree1),
                ((0.4, 0.6), 0.05, &[TileType::Grass, TileType::Grass2], ObjectType::Twig),
                ((0.25, 0.5), 0.07, &[TileType::Grass, TileType::Grass2], ObjectType::Grass),
                ((0.2, 0.9), 0.05, &[TileType::Stone], ObjectType::Stone),
                ((0.1, 0.8), 0.02, &[TileType::Stone], ObjectType::Flint),
            ],
            default_tile: TileType::Dirt
        }
    }
}