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
use crate::game::core::load::{reload_animations, reload_textures};

pub struct Lock;

pub fn system_player_input(handle: &mut RainHandle, state: &mut State) {
    let mut to_dash: Vec<Entity> = Vec::new();
    let mut to_walk: Vec<Entity> = Vec::new();
    let mut to_remove_walk: Vec<Entity> = Vec::new();
    let mut open_inventory = false;
    let mut use_item = false;
    let mut pickup_item: Option<Vec2> = None;
    let mut drop_item: Option<bool> = None;
    let mut inventory_hotbar_select: Option<usize> = None;
    let mut to_reload_textures = false;
    let mut to_reload_animations = false;

    for (e, (_, direction, position, lock)) in handle.world.query::<(&Player, &mut Direction, &Position2D, Option<&Lock>)>().iter() {
        let mut movement = Vec2::ZERO;
        if handle.is_key_pressed(KeyboardKey::A) {
            movement.x -= 1.0;
        }
        if handle.is_key_pressed(KeyboardKey::D) {
            movement.x += 1.0;
        }
        if handle.is_key_pressed(KeyboardKey::W) {
            movement.y += 1.0;
        }
        if handle.is_key_pressed(KeyboardKey::S) {
            movement.y -= 1.0;
        }

        if lock.is_none() && movement != Vec2::ZERO {
            *direction = Direction(movement.normalize());
            to_walk.push(e);
        } else {
            to_remove_walk.push(e);
        }

        if handle.is_key_pressed(KeyboardKey::M) {
            state.to_reset = true;
        }
        if handle.is_key_pressed(KeyboardKey::N) {
            to_reload_textures = true;
        }
        if handle.is_key_pressed(KeyboardKey::B) {
            to_reload_animations = true;
        }
        

        if lock.is_none() {
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
                use_item = true;
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
    }

    if to_reload_textures {
        reload_textures(handle);
    }
    if to_reload_animations {
        reload_animations(handle);
    }

    for e in to_dash {
        handle.world.insert_one(e, Dash).unwrap();
    }
    for e in to_walk {
        handle.world.insert_one(e, Walk).unwrap();
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
    if use_item {
        item_use(handle);
    }
    if let Some(drop_all) = drop_item {
        drop_current_item(handle, state, drop_all);
    }

    for e in to_remove_walk {
        let _ = handle.world.remove_one::<Walk>(e);
    }
}