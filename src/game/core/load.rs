use rain::engine::core::RainHandle;

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
    handle.load_texture("tile_dirt", "res/texture/dirt.png").expect("Error loading texture.");
    handle.load_texture("tile_grass", "res/texture/grass.png").expect("Error loading texture.");
    handle.load_texture("tile_grass2", "res/texture/grass2.png").expect("Error loading texture.");
    handle.load_texture("tile_stone", "res/texture/stone.png").expect("Error loading texture.");
    handle.load_texture("tile_cobblestone", "res/texture/cobblestone.png").expect("Error loading texture.");
    handle.load_texture("tile_water", "res/texture/water.png").expect("Error loading texture.");
    handle.load_texture("tile_sand", "res/texture/sand.png").expect("Error loading texture.");
    handle.load_texture("tile_clay", "res/texture/tile_clay.png").expect("Error loading texture.");
    handle.load_texture("tile_mud", "res/texture/tile_mud.png").expect("Error loading texture.");
    handle.load_texture("object_tree1", "res/texture/tree1.png").expect("Error loading texture.");
    handle.load_texture("object_twig", "res/texture/twig.png").expect("Error loading texture.");
    handle.load_texture("object_grass", "res/texture/object_grass.png").expect("Error loading texture.");
    handle.load_texture("object_stone", "res/texture/object_stone.png").expect("Error loading texture.");
    handle.load_texture("object_flint", "res/texture/object_flint.png").expect("Error loading texture.");
    handle.load_texture("item_twine", "res/texture/item_twine.png").expect("Error loading texture.");
    handle.load_texture("item_sling", "res/texture/item_sling.png").expect("Error loading texture.");
    handle.load_texture("item_wood", "res/texture/item_wood.png").expect("Error loading texture.");
    handle.load_texture("item_wood_planks", "res/texture/item_wood_planks.png").expect("Error loading texture.");
    handle.load_texture("item_stone_pickaxe", "res/texture/item_stone_pickaxe.png").expect("Error loading texture.");
    handle.load_texture("flint_hatchet", "res/texture/flint_hatchet.png").expect("Error loading texture.");
    handle.load_texture("health_bar_frame", "res/texture/health_bar_frame.png").expect("Error loading texture.");
    handle.load_texture("health_bar_background", "res/texture/health_bar_background.png").expect("Error loading texture.");
    handle.load_texture("inventory_slot", "res/texture/inventory_slot.png").expect("Error loading texture.");
    handle.load_texture("inventory_slot_selected", "res/texture/inventory_slot_selected.png").expect("Error loading texture.");
    handle.load_texture("player_front", "res/texture/player_front.png").expect("Error loading texture.");
    handle.load_texture("player_back", "res/texture/player_back.png").expect("Error loading texture.");
    handle.load_texture("player_side", "res/texture/player_side.png").expect("Error loading texture.");
    handle.load_texture("enemy_squirrel_side", "res/texture/enemy_squirrel_side.png").expect("Error loading texture.");
    handle.load_texture("enemy_squirrel_front", "res/texture/enemy_squirrel_front.png").expect("Error loading texture.");
    handle.load_texture("enemy_squirrel_back", "res/texture/enemy_squirrel_back.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_side", "res/texture/enemy_coati_side.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_front", "res/texture/enemy_coati_front.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_back", "res/texture/enemy_coati_back.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_crouching_side", "res/texture/enemy_coati_crouching_side.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_crouching_front", "res/texture/enemy_coati_crouching_front.png").expect("Error loading texture.");
    handle.load_texture("enemy_coati_crouching_back", "res/texture/enemy_coati_crouching_back.png").expect("Error loading texture.");
    handle.load_texture("item_coati_pelt", "res/texture/item_coati_pelt.png").expect("Error loading texture.");
    handle.load_texture("item_coati_bone", "res/texture/item_coati_bone.png").expect("Error loading texture.");
    handle.load_texture("item_coati_bone_plate", "res/texture/item_coati_bone_plate.png").expect("Error loading texture.");
    handle.load_texture("item_bone_shovel", "res/texture/item_bone_shovel.png").expect("Error loading texture.");
    handle.load_texture("item_wood_shovel", "res/texture/item_wood_shovel.png").expect("Error loading texture.");
    handle.load_texture("item_acorn", "res/texture/item_acorn.png").expect("Error loading texture.");
}

pub fn load_animations(handle: &mut RainHandle) {
    handle.load_animation("animation_player_walking_side", "res/animations/player_walking_side.json").expect("Error loading animation");
    handle.load_animation("animation_player_walking_front", "res/animations/player_walking_front.json").expect("Error loading animation");
    handle.load_animation("animation_player_walking_back", "res/animations/player_walking_back.json").expect("Error loading animation");
    handle.load_animation("animation_player_swinging_side", "res/animations/player_swinging_side.json").expect("Error loading animation");
    handle.load_animation("animation_player_swinging_front", "res/animations/player_swinging_front.json").expect("Error loading animation");
    handle.load_animation("animation_player_swinging_back", "res/animations/player_swinging_back.json").expect("Error loading animation");
    handle.load_animation("animation_flint_hatchet_swing_side", "res/animations/flint_hatchet_swing_side.json").expect("Error loading animation");
    handle.load_animation("animation_flint_hatchet_swing_front", "res/animations/flint_hatchet_swing_front.json").expect("Error loading animation");
    handle.load_animation("animation_flint_hatchet_swing_back", "res/animations/flint_hatchet_swing_back.json").expect("Error loading animation");
    handle.load_animation("animation_coati_walking_side", "res/animations/coati_walking_side.json").expect("Error loading animation");
    handle.load_animation("animation_coati_walking_front", "res/animations/coati_walking_front.json").expect("Error loading animation");
    handle.load_animation("animation_coati_walking_back", "res/animations/coati_walking_back.json").expect("Error loading animation");
}