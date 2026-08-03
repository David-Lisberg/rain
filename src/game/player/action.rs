use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::animation::{Animation, AnimationPool};
use rain::engine::component::*;
use rain::engine::core::RainHandle;
use rain::engine::input::MouseButton;
use rain::engine::texture::Texture;

use crate::State;
use crate::game::core::animation::AnimationStateUpdated;
use crate::game::core::collision::{Collider, check_collision_with_object};
use crate::game::entity::loot::roll_loot;
use crate::game::entity::projectile::{ProjectileSpawn, spawn_projectile};
use crate::game::player::inventory::Inventory;
use crate::game::player::item::*;
use crate::game::player::movement::Player;
use crate::game::utility::direction::Direction4;
use crate::game::world::object::{ObjectType, destroy_object, reload_object_mesh};


struct SlingHold(f32, usize);

#[derive(Debug, PartialEq)]
pub enum AnimationStatePlayer {
    None,
    Walking(Direction4),
    Attacking(Direction4, Option<Item>),
}

pub fn item_attack(handle: &mut RainHandle, state: &mut State, direction: Vec2) {
    let mut object_changed = false;
    let mut to_add_updated: Vec<Entity> = Vec::new();
    let mut to_spawn_item_drop: Vec<(Position2D, Item, i32)> = Vec::new();
    
    for (e, (_, position, inventory, player_direction, animation_state)) in handle.world.query_mut::<(
        &Player, &Position2D, &mut Inventory, &mut Direction, &mut AnimationStatePlayer
    )>() {
        let collider_position = position.0 + direction;
        let collider = Collider::from_center(collider_position.x, collider_position.y, 1.0, 1.0);
        let direction4 = Direction4::from_vec2(direction);
        
        let (tool_type, break_level, hit_ticks) = if let Some(item) = &inventory.slots[inventory.selected_hotbar].item {
            match item.category {
                ItemCategory::Tool(t, b, h, _) => (t, b, h),
                _ => (ToolType::None, 0, 1),
            }
        } else {
            (ToolType::None, 0, 1)
        };
        *player_direction = Direction(direction);
        *animation_state = AnimationStatePlayer::Attacking(direction4, inventory.slots[inventory.selected_hotbar].item.clone());
        to_add_updated.push(e);
        
        if let Some(object) = check_collision_with_object(state, &collider) {
            if break_level >= object.break_level && tool_type.can_break(object.required_tool) {
                if destroy_object(state, &object, hit_ticks) {
                    let data = state.object_registry.get(&object._type).unwrap();
                    let drops = roll_loot(state, &data.loot_table.clone());
                    for (item, quantity) in drops {
                        let remaining = inventory.add_item(item.clone(), quantity);
                        if remaining > 0 {
                            to_spawn_item_drop.push((Position2D(object.center()), item, remaining));
                        }
                    }
                    object_changed = true;
                }
            }
        }
    }
    for (position, item, quantity) in to_spawn_item_drop {
        spawn_item_drop(handle, state, position, item, quantity);
    }
    if object_changed {
        reload_object_mesh(handle, state);
    }
    for e in to_add_updated {
        handle.world.insert_one(e, AnimationStateUpdated).unwrap();
    }
}

pub fn item_use(handle: &mut RainHandle) {
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
            *flip = Flip(false, false);
            *texture = player_back.clone();
        } else if direction.0.y < -0.8 {
            *flip = Flip(false, false);
            *texture = player_front.clone();
        } else if direction.0.x.is_sign_positive() {
            *flip = Flip(false, false);
            *texture = player_side.clone();
        } else if direction.0.x.is_sign_negative() {
            *flip = Flip(true, false);
            *texture = player_side.clone();
        }
    }
}

pub fn system_update_player_animation(handle: &mut RainHandle) {
    let mut to_add_animation: Vec<(Entity, Animation)> = Vec::new();
    let mut to_add_animation_pool: Vec<(Entity, Animation, usize)> = Vec::new();
    let mut to_remove_updated: Vec<Entity> = Vec::new();
    let mut to_remove_animations: Vec<Entity> = Vec::new();

    for (e, (_, state, _)) in handle.world.query::<(&Player, &AnimationStatePlayer, &AnimationStateUpdated)>().iter() {
        to_remove_updated.push(e);
        match state {
            AnimationStatePlayer::None => to_remove_animations.push(e),
            AnimationStatePlayer::Walking(direction) => {
                match direction {
                    Direction4::N => to_add_animation.push((e, Animation::new("animation_player_walking_back"))),
                    Direction4::S => to_add_animation.push((e, Animation::new("animation_player_walking_front"))),
                    Direction4::E | Direction4::W => to_add_animation.push((e, Animation::new("animation_player_walking_side"))),
                }
            }
            AnimationStatePlayer::Attacking(direction, item) => {
                let (item_type, item_category) = match item {
                    Some(i) => (Some(i._type.clone()), i.category.clone()),
                    None => (None, ItemCategory::Other)
                };
                let direction_string = match direction {
                    Direction4::N => "_back",
                    Direction4::S => "_front",
                    Direction4::E | Direction4::W => "_side",
                };

                match item_category {
                    ItemCategory::Tool(_, _, _, _) => {
                        let item_type = item_type.unwrap();
                        let item_string = match item_type {
                            ItemType::FlintHatchet => Some("_flint_hatchet_swing"),
                            ItemType::BoneHatchet => Some("_bone_hatchet_swing"),
                            _ => None,
                        };
                        if let Some(string) = item_string {
                            to_add_animation_pool.push((e, Animation::new(&format!("{}{}{}", "animation", string, direction_string)), 0));
                            to_add_animation.push((e, Animation::new(&format!("{}{}", "animation_player_swinging", direction_string))))
                        } else {
                            to_add_animation.push((e, Animation::new(&format!("{}{}", "animation_player_punching", direction_string))));
                        }
                    }
                    ItemCategory::Other => {
                        to_add_animation.push((e, Animation::new(&format!("{}{}", "animation_player_punching", direction_string))))
                    }
                }
            }
        }
    }

    for (e, animation) in to_add_animation {
        handle.world.insert_one(e, animation).unwrap();
    }
    for (e, animation, key) in to_add_animation_pool {
        if let Ok(pool) = handle.world.query_one_mut::<&mut AnimationPool>(e) {
            pool.animations.insert(key, animation);
        }
    }
    for e in to_remove_updated {
        handle.world.remove_one::<AnimationStateUpdated>(e).unwrap();
    }
    for e in to_remove_animations {
        handle.world.remove_one::<Animation>(e).unwrap();
        if let Ok(pool) = handle.world.query_one_mut::<&mut AnimationPool>(e) {
            pool.animations.clear();
        }
    }
}

pub fn system_clear_animation_state(handle: &mut RainHandle) {
    for (_, (_, state, animation, animation_pool)) in handle.world.query_mut::<(
        &Player, &mut AnimationStatePlayer, Option<&Animation>, Option<&AnimationPool>
    )>() {
        if animation.is_some() {
            continue;
        }
        if let Some(pool) = animation_pool {
            if !pool.animations.is_empty() {
                continue;
            }
        }
        *state = AnimationStatePlayer::None;
    }
}

pub fn system_player_action(handle: &mut RainHandle) {
    system_player_sling(handle);
}

fn system_player_sling(handle: &mut RainHandle) {
    let pressed = handle.is_button_pressed(MouseButton::Right);
    let mut sling_released: Option<(Entity, Vec2)> = None;
    let mut sling_cancel: Option<Entity> = None;
    let mut player_entity: Option<Entity> = None;

    for (e, (_, inventory, position, sling_hold)) in handle.world.query_mut::<(&Player, &mut Inventory, &Position2D, &mut SlingHold)>() {
        player_entity = Some(e);
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
        let spawn = ProjectileSpawn::new(
            player_entity.unwrap(), "object_stone".to_string(), sling_hold.0.min(1.5) * 28.0, direction, position, Vec2::new(0.4, 0.4), 10.0
        );

        spawn_projectile(handle, spawn);
    }
}