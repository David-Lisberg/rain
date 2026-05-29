use serde::Deserialize;

use crate::State;
use crate::game::player::inventory::Inventory;
use crate::game::player::item::{Item, ItemType};

pub type RecipeRegistry = Vec<Recipe>;
#[derive(Clone, Debug, Deserialize)]
pub struct Recipe {
    pub input: Vec<(ItemType, i32)>,
    pub output: (ItemType, i32),
}

pub fn check_available_recipes(state: &mut State, inputs: &Vec<(ItemType, i32)>) -> Vec<Recipe> {
    let mut available_recipes: Vec<Recipe> = Vec::new();

    for recipe in &state.recipe_registry {
        let mut i = 0;
        loop {
            let mut failed = true;
            for input in inputs {
                if recipe.input[i].0 == input.0 && recipe.input[i].1 <= input.1 {
                    i += 1;
                    failed = false;
                    break;
                }
            }
            if i >= recipe.input.len() {
                available_recipes.push(recipe.clone());
                break;
            }
            if failed {
                break;
            }
        }
    }

    available_recipes
}

pub fn craft_item(inventory: &mut Inventory, recipe: &Recipe) {
    for input in &recipe.input {
        let mut remaining = input.1;
        for slot in inventory.slots.iter_mut() {
            if let Some(item) = &slot.item {
                if item._type == input.0 {
                    if slot.quantity >= remaining {
                        slot.quantity -= remaining;
                        remaining = 0;
                        if slot.quantity == 0 {
                            slot.item = None;
                        }
                        break;
                    } else {
                        remaining -= slot.quantity;
                        slot.quantity = 0;
                        slot.item = None;
                    }
                }
            }
        }
        if remaining > 0 {
            return;
        }
    }
    inventory.add_item(Item::new(recipe.output.0.clone()), recipe.output.1);
}