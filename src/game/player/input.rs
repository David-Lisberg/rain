use glam::Vec2;
use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rain::engine::input::KeyboardKey;

use crate::State;
use crate::game::player::action::item_pickup;
use crate::game::player::inventory::Inventory;
use crate::game::player::movement::Player;


pub fn system_player_input(handle: &mut RainHandle, state: &mut State) {
    let mut to_dash: Vec<Entity> = Vec::new();
    let mut to_walk: Vec<Entity> = Vec::new();
    let mut to_remove_walk: Vec<Entity> = Vec::new();
    let mut open_inventory = false;
    let mut pickup_item = false;
    for (e, (_, direction)) in handle.world.query::<(&Player, &mut Direction)>().iter() {
        if handle.is_key_pressed(KeyboardKey::A) && handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction(Vec2::new(-1.0, 1.0).normalize());
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) && handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction(Vec2::new(1.0, 1.0).normalize());
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) && handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction(Vec2::new(1.0, -1.0).normalize());
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) && handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction(Vec2::new(-1.0, -1.0).normalize());
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction(Vec2::new(-1.0, 0.0));
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction(Vec2::new(1.0, 0.0));
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction(Vec2::new(0.0, -1.0));
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction(Vec2::new(0.0, 1.0));
            to_walk.push(e);
        } else {
            to_remove_walk.push(e);
        }
        if handle.is_key_released(KeyboardKey::Space) {
            to_dash.push(e);
        }
        if handle.is_key_released(KeyboardKey::E) {
            open_inventory = true;
        }
        if handle.is_key_released(KeyboardKey::Q) {
            pickup_item = true;
        }
    }

    for e in to_dash {
        handle.world.insert_one(e, Dash).unwrap();
    }
    for e in to_walk {
        handle.world.insert_one(e, Walk).unwrap();
    }
    for e in to_remove_walk {
        let _ = handle.world.remove_one::<Walk>(e);
    }
    if open_inventory {
        for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
            inventory.open = !inventory.open;
        }
    }
    if pickup_item {
        item_pickup(handle, state);
    }
}
