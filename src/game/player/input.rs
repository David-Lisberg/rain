use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rain::engine::input::KeyboardKey;

use crate::game::player::inventory::Inventory;
use crate::game::player::movement::Player;


pub fn system_player_input(handle: &mut RainHandle) {
    let mut to_dash: Vec<Entity> = Vec::new();
    let mut to_walk: Vec<Entity> = Vec::new();
    let mut to_remove_walk: Vec<Entity> = Vec::new();
    let mut open_inventory = false;
    for (e, (_, direction)) in handle.world.query::<(&Player, &mut Direction)>().iter() {
        if handle.is_key_pressed(KeyboardKey::A) && handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction::UpLeft;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) && handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction::UpRight;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) && handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction::DownRight;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) && handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction::DownLeft;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction::Left;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction::Right;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction::Down;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction::Up;
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
}
