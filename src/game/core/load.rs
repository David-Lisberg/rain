use rain::engine::core::RainHandle;

use crate::game::player::item::ItemRegistry;

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

pub fn load_registry(path: &str) -> ItemRegistry {
    let json = std::fs::read_to_string(path).expect("Error loading item registry.");
    serde_json::from_str(&json).expect("Error parsing item registry.")
}