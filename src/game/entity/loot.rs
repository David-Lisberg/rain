use std::ops::RangeInclusive;

use rand::RngExt;

use crate::{State, game::player::item::Item};

pub struct LootTable {
    pub drops: Vec<(f32, RangeInclusive<i32>, Item)>
}

pub fn roll_loot(state: &mut State, loot_table: &LootTable) -> Vec<(Item, i32)> {
    let mut loot: Vec<(Item, i32)> = Vec::new();

    for (chance, quantity_range, item) in &loot_table.drops {
        let random_value = state.rng.random::<f32>();
        if random_value <= *chance {
            let quantity = state.rng.random_range(quantity_range.clone());
            loot.push((item.clone(), quantity));
        }
    }

    loot
}
