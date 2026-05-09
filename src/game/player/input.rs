use glam::Vec2;
use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rain::engine::input::{KeyboardKey, MouseButton};

use crate::State;
use crate::game::player::action::{item_attack, item_use};
use crate::game::player::inventory::Inventory;
use crate::game::player::item::drop_current_item;
use crate::game::player::movement::Player;


pub fn system_player_input(handle: &mut RainHandle, state: &mut State) {
    let mut to_dash: Vec<Entity> = Vec::new();
    let mut to_walk: Vec<Entity> = Vec::new();
    let mut to_remove_walk: Vec<Entity> = Vec::new();
    let mut open_inventory = false;
    let mut pickup_item: Option<Vec2> = None;
    let mut use_item: Option<Vec2> = None;
    let mut drop_item: Option<bool> = None;
    let mut inventory_hotbar_select: Option<usize> = None;
    for (e, (_, direction, position)) in handle.world.query::<(&Player, &mut Direction, &Position2D)>().iter() {
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
            if handle.is_key_pressed(KeyboardKey::ControlLeft) {
                drop_item = Some(true);
            } else {
                drop_item = Some(false);
            }
        }
        if handle.is_button_released(MouseButton::Left) {
            pickup_item = Some(position.0.clone());
        }
        if handle.is_button_just_pressed(MouseButton::Right) {
            use_item = Some(direction.0.clone());
        }
        if handle.is_key_released(KeyboardKey::Digit1) {
            inventory_hotbar_select = Some(0);
        } else if handle.is_key_released(KeyboardKey::Digit2) {
            inventory_hotbar_select = Some(1);
        } else if handle.is_key_released(KeyboardKey::Digit3) {
            inventory_hotbar_select = Some(2);
        } else if handle.is_key_released(KeyboardKey::Digit4) {
            inventory_hotbar_select = Some(3);
        } else if handle.is_key_released(KeyboardKey::Digit5) {
            inventory_hotbar_select = Some(4);
        } else if handle.is_key_released(KeyboardKey::Digit6) {
            inventory_hotbar_select = Some(5);
        } else if handle.is_key_released(KeyboardKey::Digit7) {
            inventory_hotbar_select = Some(6);
        } else if handle.is_key_released(KeyboardKey::Digit8) {
            inventory_hotbar_select = Some(7);
        } else if handle.is_key_released(KeyboardKey::Digit9) {
            inventory_hotbar_select = Some(8);
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
            if !inventory.open {
                inventory.selected.clear();
            } else {
                inventory.just_opened = true;
            }
        }
    }
    if let Some(index) = inventory_hotbar_select {
        for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
            if !inventory.open {
                inventory.selected_hotbar = index;
            }
        }
    }
    if let Some(position) = pickup_item {
        let mouse_position = handle.screen_position_to_world_position(handle.mouse_position());
        let direction = (mouse_position - position).normalize();
        item_attack(handle, state, direction);
    }
    if let Some(direction) = use_item {
        item_use(handle, state, direction);
    }
    if let Some(drop_all) = drop_item {
        drop_current_item(handle, drop_all);
    }
}