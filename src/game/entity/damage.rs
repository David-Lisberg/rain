use hecs::Entity;
use rain::engine::{component::Position2D, core::RainHandle};

use crate::{State, game::{core::collision::Collider, entity::enemy::{Enemy, EnemyType}, player::item::{Item, ItemType, spawn_item_drop}}};

pub struct HurtBox(pub Collider);
#[derive(Clone)]
pub struct HitBox {
    pub damage: f32,
    pub collider: Collider,
    pub safe: Vec<Entity>,
    pub uses: i32,
}

pub struct Health(pub f32);

pub fn system_hitbox_hurtbox_collision(handle: &mut RainHandle, state: &mut State) {
    let mut hitboxes: Vec<(Entity, HitBox)> = Vec::new();
    let mut to_kill: Vec<Entity> = Vec::new();
    let mut to_remove: Vec<Entity> = Vec::new();
    let mut to_spawn: Vec<(Position2D, Item, i32)> = Vec::new();
    for (e, hitbox) in handle.world.query::<&HitBox>().iter() {
        hitboxes.push((e, hitbox.clone()));
    }
    for (e, (hurtbox, health)) in handle.world.query_mut::<(&HurtBox, &mut Health)>() {
        for (other_e, hitbox) in hitboxes.iter_mut() {
            if e == *other_e || hitbox.safe.contains(&e) {
                continue;
            }
            if hitbox.uses > 0 && hitbox.collider.aabb_collision(&hurtbox.0) {
                hitbox.uses -= 1;
                if hitbox.uses <= 0 {
                    to_remove.push(*other_e);
                }
                health.0 -= hitbox.damage;
                if health.0 <= 0.0 {
                    to_kill.push(e);
                    break;
                }
                
            }
        }
    }
    for e in to_kill {
        if let Some((enemy, position)) = handle.world.query_one::<(&Enemy, &Position2D)>(e).unwrap().get() {
            match enemy._type {
                EnemyType::Coati => {
                    to_spawn.push((position.clone(), Item::new(ItemType::CoatiPelt), 2));
                }
            }
            state.enemy_count -= 1;
        }
        to_remove.push(e);
    }
    for (position, item, quantity) in to_spawn {
        spawn_item_drop(handle, position, item, quantity);
    }
    for e in to_remove {
        if handle.world.get::<&Enemy>(e).is_ok() {
            state.enemy_count -= 1;
        }
        handle.world.despawn(e).unwrap();
    }
}