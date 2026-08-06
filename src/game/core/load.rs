use std::collections::HashMap;

use rain::engine::core::RainHandle;

use crate::game::entity::enemy::EnemyRegistry;
use crate::game::player::crafting::RecipeRegistry;
use crate::game::player::item::ItemRegistry;
use crate::game::world::config::WorldGenConfig;
use crate::game::world::object::{ObjectData, ObjectDataRaw, ObjectRegistry, ObjectType};

type TextureRegistry = Vec<(String, String)>;
type AnimationRegistry = Vec<(String, String)>;

pub fn reload_textures(handle: &mut RainHandle) {
    let default = handle.fetch_texture("").unwrap();
    handle.resource_manager.textures.clear();
    handle.resource_manager.textures.insert("".to_string(), default);
    load_textures(handle);
}

pub fn reload_animations(handle: &mut RainHandle) {
    handle.resource_manager.animations.clear();
    load_animations(handle);
}

pub fn load_textures(handle: &mut RainHandle) {
    let json = std::fs::read_to_string("res/assets/textures.json").expect("Error loading texture registry.");
    let texture_registry: TextureRegistry = serde_json::from_str(&json).expect("Error parsing texture registry.");
    
    for (name, path) in texture_registry {
        handle.load_texture(&name, &path).expect("Error loading animation");
    }
}

pub fn load_animations(handle: &mut RainHandle) {
    let json = std::fs::read_to_string("res/assets/animations.json").expect("Error loading animation registry.");
    let animation_registry: AnimationRegistry = serde_json::from_str(&json).expect("Error parsing animation registry.");
    
    for (name, path) in animation_registry {
        handle.load_animation(&name, &path).expect("Error loading animation");
    }
}

pub fn load_item_registry(path: &str) -> ItemRegistry {
    let json = std::fs::read_to_string(path).expect("Error loading item registry.");
    serde_json::from_str(&json).expect("Error parsing item registry.")
}

pub fn load_recipe_registry(path: &str) -> RecipeRegistry {
    let json = std::fs::read_to_string(path).expect("Error loading recipe registry.");
    serde_json::from_str(&json).expect("Error parsing recipe registry.")
}

pub fn load_enemy_registry(path: &str) -> EnemyRegistry {
    let json = std::fs::read_to_string(path).expect("Error loading enemy registry.");
    serde_json::from_str(&json).expect("Error parsing enemy registry.")
}

pub fn load_object_registry(handle: &mut RainHandle, path: &str) -> ObjectRegistry {
    let json = std::fs::read_to_string(path).expect("Error loading object registry.");
    let raw: HashMap<ObjectType, ObjectDataRaw> = serde_json::from_str(&json).expect("Error parsing object registry.");
    raw.into_iter()
        .map(|(key, raw_data)| (key, ObjectData::from_raw(handle, raw_data)))
        .collect()
}

pub fn load_world_gen_config(path: &str) -> WorldGenConfig {
    let json = std::fs::read_to_string(path).expect("Error loading world gen config.");
    serde_json::from_str(&json).expect("Error parsing world gen config.")
}