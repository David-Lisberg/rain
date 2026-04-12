use std::sync::Arc;

use glam::Vec2;
use rain::engine::{component::*, core::RainHandle, texture::Texture};

use crate::{State, game::{core::collision::*, player::{inventory::Inventory, item::*, movement::Player}, world::object::{ObjectType, destroy_object, reload_object_mesh}}};

pub fn item_pickup(handle: &mut RainHandle, state: &mut State, direction: Vec2) {
    let mut object_changed = false;
    let query = handle.world.query_mut::<(&Player, &Position2D, &mut Inventory)>();
    for (_, (_, position, inventory)) in query {
        let collider_position = position.0 + direction;
        let collider = Collider::from_center(collider_position.x, collider_position.y, 1.0, 1.0);
        if let Some(object) = check_collision_with_object(state, &collider) {
            match object._type {
                ObjectType::Twig => {
                    if destroy_object(state, &object) {
                        object_changed = true;
                        inventory.add_item(Item::new(ItemType::Twig), 1);
                    }
                }
                ObjectType::Grass => {
                    if destroy_object(state, &object) {
                        object_changed = true;
                        inventory.add_item(Item::new(ItemType::Grass), 1);
                    }
                }
                ObjectType::Stone => {
                    if destroy_object(state, &object) {
                        object_changed = true;
                        inventory.add_item(Item::new(ItemType::Stone), 1);
                    }
                }
                _ => {}
            }
        }
    }

    if object_changed {
        reload_object_mesh(handle, state);
    }
}

pub fn system_update_player_texture(handle: &mut RainHandle) {
    let player_front = handle.fetch_texture("player_front").unwrap();
    let player_back = handle.fetch_texture("player_back").unwrap();
    let player_side = handle.fetch_texture("player_side").unwrap();
    for (_, (_, direction, texture, flip)) in handle.world.query_mut::<(&Player, &Direction, &mut Arc<Texture>, &mut Flip)>() {
        if direction.0.y > 0.8 {
            *texture = player_back.clone();
            *flip = Flip(false, false);
        } else if direction.0.y < -0.8 {
            *texture = player_front.clone();
            *flip = Flip(false, false);
        } else if direction.0.x.is_sign_positive() {
            *texture = player_side.clone();
            *flip = Flip(false, false);
        } else if direction.0.x.is_sign_negative() {
            *texture = player_side.clone();
            *flip = Flip(true, false);
        }
    }
}