use rand::RngExt;
use serde::Deserialize;

use crate::State;
use crate::game::player::item::Item;

#[derive(Deserialize, Clone)]
pub struct LootTableEntry {
    pub chance: f32,
    pub min: i32,
    pub max: i32,
    pub item: Item,
}

#[derive(Deserialize, Clone)]
pub struct LootTable(pub Vec<LootTableEntry>);

pub fn roll_loot(state: &mut State, loot_table: &LootTable) -> Vec<(Item, i32)> {
    let mut loot: Vec<(Item, i32)> = Vec::new();

    for entry in &loot_table.0 {
        let random_value = state.rng.random::<f32>();
        if random_value <= entry.chance {
            let quantity = state.rng.random_range(entry.min..=entry.max);
            loot.push((entry.item.clone(), quantity));
        }
    }

    loot
}
