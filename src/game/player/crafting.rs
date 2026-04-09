use crate::game::player::{inventory::Inventory, item::{Item, ItemType}};

#[derive(Clone, Debug)]
pub struct Recipe {
    pub input: &'static [(ItemType, u32)],
    pub output: (ItemType, u32),
}

const AVAILABLE_RECIPES: &[Recipe] = &[
    Recipe { input: &[(ItemType::Grass, 2)], output: (ItemType::Twine, 1)},
    Recipe { input: &[(ItemType::Twine, 3), (ItemType::Twig, 2)], output: (ItemType::Sling, 1)},
];

pub fn check_available_recipes(inputs: &Vec<(ItemType, u32)>) -> Vec<Recipe> {
    let mut available_recipes: Vec<Recipe> = Vec::new();

    for recipe in AVAILABLE_RECIPES {
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
    for input in recipe.input {
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
    inventory.add_item(Item { _type: recipe.output.0.clone() }, recipe.output.1);
}