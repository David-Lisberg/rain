use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{animation::{Animation, AnimationPool}, component::*, core::RainHandle, input::MouseButton, texture::Texture};

use crate::{DEPTH_PROJECTILE, State, game::{core::collision::*, entity::{damage::HitBox, despawn::TimerDespawn, projectile::Projectile}, player::{inventory::Inventory, item::*, movement::Player}, utility::{direction::Direction8, timer::Timer}, world::object::{ObjectType, destroy_object, reload_object_mesh}}};

struct SlingHold(f32, usize);
pub struct PlayerAttacking;

pub fn item_attack(handle: &mut RainHandle, state: &mut State, direction: Vec2) {
    let mut object_changed = false;
    let mut to_add_animation_pool: Vec<(Entity, Animation)> = Vec::new();
    let mut to_add_animation: Vec<(Entity, Animation)> = Vec::new();
    let mut to_add_attacking: Vec<Entity> = Vec::new();
    let mut to_spawn_item_drop: Vec<(Position2D, Item, i32)> = Vec::new();
    
    for (e, (_, position, inventory, player_direction)) in handle.world.query_mut::<(
        &Player, &Position2D, &mut Inventory, &mut Direction
    )>() {
        let collider_position = position.0 + direction;
        let collider = Collider::from_center(collider_position.x, collider_position.y, 1.0, 1.0);
        let direction8 = Direction8::from_vec2_8way(direction);
        let (tool_type, break_level, hit_ticks) = if let Some(item) = &inventory.slots[inventory.selected_hotbar].item {
            match direction8 {
                Direction8::E | Direction8::NE | Direction8::NW | Direction8::SW | Direction8::SE | Direction8::W => {
                    match item._type {
                        ItemType::FlintHatchet => to_add_animation_pool.push((e, Animation::new("animation_flint_hatchet_swing_side"))),
                        _ => {}
                    }
                    to_add_animation.push((e, Animation::new("animation_player_swinging_side")));
                }
                Direction8::N => {
                    match item._type {
                        ItemType::FlintHatchet => to_add_animation_pool.push((e, Animation::new("animation_flint_hatchet_swing_back"))),
                        _ => {}
                    }
                    to_add_animation.push((e, Animation::new("animation_player_swinging_back")));
                }
                Direction8::S => {
                    match item._type {
                        ItemType::FlintHatchet => to_add_animation_pool.push((e, Animation::new("animation_flint_hatchet_swing_front"))),
                        _ => {}
                    }
                    to_add_animation.push((e, Animation::new("animation_player_swinging_front")));
                }
            }

            match item.category {
                ItemCategory::Tool(t, b, h, _) => (t, b, h),
                _ => (ToolType::None, 0, 1),
            }
        } else {
            (ToolType::None, 0, 1)
        };
        *player_direction = Direction(direction);
        
        if let Some(object) = check_collision_with_object(state, &collider) {
            if break_level >= object.break_level && tool_type.can_break(object.required_tool) {
                if destroy_object(state, &object, hit_ticks) {
                    let (item, quantity) = match object._type {
                        ObjectType::Twig => (Item::new(ItemType::Twig), 1),
                        ObjectType::Grass => (Item::new(ItemType::Grass), 1),
                        ObjectType::Stone => (Item::new(ItemType::Stone), 1),
                        ObjectType::Flint => (Item::new(ItemType::Flint), 1),
                        ObjectType::Tree1 => (Item::new(ItemType::Wood), 3),
                    };
                    object_changed = true;
                    let remaining = inventory.add_item(item.clone(), quantity);
                    if remaining > 0 {
                        to_spawn_item_drop.push((Position2D(object.center()), item, remaining));
                    }
                }
            }
        }
    }
    for (position, item, quantity) in to_spawn_item_drop {
        spawn_item_drop(handle, position, item, quantity);
    }
    if object_changed {
        reload_object_mesh(handle, state);
    }
    for (e, animation) in to_add_animation_pool {
        if let Ok(pool) = handle.world.query_one_mut::<&mut AnimationPool>(e) {
            pool.animations.insert(0, animation);
        }
        to_add_attacking.push(e);
    }
    for (e, animation) in to_add_animation {
        handle.world.insert_one(e, animation).unwrap();
    }
    for e in to_add_attacking {
        handle.world.insert_one(e, PlayerAttacking).unwrap();
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
    for (_, (_, direction, animation, texture, flip)) in handle.world.query_mut::<(&Player, &Direction, Option<&Animation>, &mut Arc<Texture>, &mut Flip)>() {
        if direction.0.y > 0.8 {
            *flip = Flip(false, false);
        } else if direction.0.y < -0.8 {
            *flip = Flip(false, false);
        } else if direction.0.x.is_sign_positive() {
            *flip = Flip(false, false);
        } else if direction.0.x.is_sign_negative() {
            *flip = Flip(true, false);
        }
        if animation.is_none() {
            if direction.0.y > 0.8 {
                *texture = player_back.clone();
            } else if direction.0.y < -0.8 {
                *texture = player_front.clone();
            } else if direction.0.x.is_sign_positive() {
                *texture = player_side.clone();
            } else if direction.0.x.is_sign_negative() {
                *texture = player_side.clone();
            }
        }
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
        let velocity = Velocity2D(direction * sling_hold.0.min(1.5) * 28.0);
        let hitbox = HitBox::new(
            10.0,
            Collider::from_center(position.x, position.y, 0.4, 0.4),
            vec![player_entity.unwrap()],
            1,
        );

        let texture = handle.fetch_texture("object_stone").unwrap();
        handle.world.spawn((
            Sprite, Visible, Position2D(position), velocity, Acceleration2D(Vec2::ZERO), TimerDespawn(Timer::new(5.0)),
            texture, Scale2D(Vec2::new(0.4, 0.4)), DepthZ(DEPTH_PROJECTILE), Priority(1), Projectile,
            hitbox,
        ));
    }
}