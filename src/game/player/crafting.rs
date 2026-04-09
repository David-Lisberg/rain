use crate::game::player::item::ItemType;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub input: &'static [(ItemType, u32)],
    pub output: (ItemType, u32),
}

const AVAILABLE_RECIPES: &[Recipe] = &[
    Recipe { input: &[(ItemType::Grass, 2)], output: (ItemType::Twine, 1)},
];

pub fn check_available_recipes(inputs: &Vec<(ItemType, u32)>) -> Vec<Recipe> {
    let mut available_recipes: Vec<Recipe> = Vec::new();

    for recipe in AVAILABLE_RECIPES {
        let mut i = 0;
        let mut failed = true;
        loop {
            for input in inputs {
                if recipe.input[i].0 == input.0 && recipe.input[i].1 <= input.1 {
                    i += 1;
                    failed = false;
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