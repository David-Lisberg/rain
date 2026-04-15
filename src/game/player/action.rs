use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::*, core::RainHandle, input::MouseButton, texture::Texture};

use crate::{DEPTH_PROJECTILE, State, game::{core::collision::*, entity::lifetime::Lifetime, player::{inventory::Inventory, item::*, movement::Player}, world::object::{ObjectType, destroy_object, reload_object_mesh}}};

struct SlingHold(f32, usize);

pub fn item_pickup(handle: &mut RainHandle, state: &mut State, direction: Vec2) {
    let mut object_changed = false;
    let query = handle.world.query_mut::<(&Player, &Position2D, &mut Inventory)>();
    for (_, (_, position, inventory)) in query {
        let collider_position = position.0 + direction;
        let collider = Collider::from_center(collider_position.x, collider_position.y, 1.0, 1.0);
        if let Some(object) = check_collision_with_object(state, &collider) {
            let (break_level, hit_ticks) = if let Some(item) = &inventory.slots[inventory.selected_hotbar].item {
                match item.category {
                    ItemCategory::Tool(b, h) => (b, h),
                    _ => (0, 1),
                }
            } else {
                (0, 1)
            };
            if break_level >= object.break_level {
                match object._type {
                    ObjectType::Twig => {
                        if destroy_object(state, &object, hit_ticks) {
                            object_changed = true;
                            inventory.add_item(Item::new(ItemType::Twig), 1);
                        }
                    }
                    ObjectType::Grass => {
                        if destroy_object(state, &object, hit_ticks) {
                            object_changed = true;
                            inventory.add_item(Item::new(ItemType::Grass), 1);
                        }
                    }
                    ObjectType::Stone => {
                        if destroy_object(state, &object, hit_ticks) {
                            object_changed = true;
                            inventory.add_item(Item::new(ItemType::Stone), 1);
                        }
                    }
                    ObjectType::Flint => {
                        if destroy_object(state, &object, hit_ticks) {
                            object_changed = true;
                            inventory.add_item(Item::new(ItemType::Flint), 1);
                        }
                    }
                    ObjectType::Tree1 => {
                        if destroy_object(state, &object, hit_ticks) {
                            object_changed = true;
                            inventory.add_item(Item::new(ItemType::Wood), 3);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if object_changed {
        reload_object_mesh(handle, state);
    }
}

pub fn item_use(handle: &mut RainHandle, state: &mut State, direction: Vec2) {
    let mut pending_use: Option<(ItemType, Entity, usize)> = None;
    for (e, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
        let slot = inventory.slots.get(inventory.selected_hotbar).unwrap();
        if let Some(item) = &slot.item {
            match item._type {
                ItemType::Sling => {
                    if inventory.search_item(ItemType::Stone, 1).is_some() {
                        pending_use = Some((ItemType::Sling, e, inventory.selected_hotbar));
                    }
                }
                _ => {}
            }
        }
    }

    if let Some((_type, e, slot)) = pending_use {
        handle.world.insert_one(e, SlingHold(0.0, slot)).unwrap();
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

pub fn system_player_action(handle: &mut RainHandle, state: &mut State) {
    system_player_sling(handle, state);
}

fn system_player_sling(handle: &mut RainHandle, state: &mut State) {
    let pressed = handle.is_button_pressed(MouseButton::Right);
    let mut sling_released: Option<(Entity, Vec2)> = None;
    let mut sling_cancel: Option<Entity> = None;

    for (e, (_, inventory, position, sling_hold)) in handle.world.query_mut::<(&Player, &mut Inventory, &Position2D, &mut SlingHold)>() {
        if inventory.selected_hotbar != sling_hold.1 || inventory.open {
            sling_cancel = Some(e);
            break;
        }

        if pressed {
            sling_hold.0 += handle.delta_time;
        } else if inventory.remove_item(ItemType::Stone, 1) {
            sling_released = Some((e, position.0.clone()));
        } else {
            sling_cancel = Some(e);
            break;
        }
    }

    if let Some(e) = sling_cancel {
        handle.world.remove_one::<SlingHold>(e).unwrap();
        return;
    }

    if let Some((e, position)) = sling_released {
        let sling_hold = handle.world.remove_one::<SlingHold>(e).unwrap();
        let mouse_position = handle.screen_position_to_world_position(handle.mouse_position());
        let direction = (mouse_position - position).normalize();
        let velocity = Velocity2D(direction * sling_hold.0.min(1.5) * 28.0);
        let texture = handle.fetch_texture("object_stone").unwrap();
        handle.world.spawn((
            Sprite, Visible, Position2D(position), velocity, Acceleration2D(Vec2::ZERO), 
            Lifetime(5.0), texture, Scale2D(Vec2::new(0.4, 0.4)), DepthZ(DEPTH_PROJECTILE), Priority(1)
        ));
    }
}