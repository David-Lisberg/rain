use hecs::Entity;
use rain::engine::{component::Position2D, core::RainHandle};

use crate::{State, game::{core::collision::Collider, entity::enemy::{Enemy, EnemyType}, player::{item::{Item, ItemType, spawn_item_drop}, movement::Player}, utility::timer::Timer}};

pub struct HurtBox(pub Collider);
#[derive(Clone)]
pub struct HitBox {
    pub damage: f32,
    pub collider: Collider,
    pub safe: Vec<Entity>,
    pub uses: i32,
}

impl HitBox {
    pub fn new(damage: f32, collider: Collider, safe: Vec<Entity>, uses: i32) -> Self {
        Self { damage, collider, safe, uses }
    }
}

pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { max, current: max }
    }
}

pub struct HealthBar(pub Entity, pub Timer, pub f32);

pub fn system_hitbox_hurtbox_collision(handle: &mut RainHandle, state: &mut State) {
    let mut hitboxes: Vec<(Entity, HitBox)> = Vec::new();
    let mut to_kill: Vec<Entity> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();
    let mut to_remove_hitbox: Vec<Entity> = Vec::new();
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
                    to_remove_hitbox.push(*other_e);
                }
                health.current -= hitbox.damage;
                if health.current <= 0.0 {
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
                    to_spawn.push((position.clone(), Item::new(ItemType::CoatiBone), 3));
                }
            }
            state.enemy_count -= 1;
        }
        to_despawn.push(e);
    }
    for (position, item, quantity) in to_spawn {
        spawn_item_drop(handle, position, item, quantity);
    }
    for e in to_despawn {
        if handle.world.get::<&Enemy>(e).is_ok() {
            state.enemy_count -= 1;
        }
        if handle.world.get::<&Player>(e).is_ok() {
            state.to_reset = true;
            break;
        }
        handle.world.despawn(e).unwrap();
    }
    for e in to_remove_hitbox {
        handle.world.remove_one::<HitBox>(e).unwrap();
    }
}

pub fn system_health_bar(handle: &mut RainHandle) {
    let mut parents: Vec<(Entity, Entity)> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (e, health_bar) in handle.world.query::<&HealthBar>().iter() {
        parents.push((e, health_bar.0));
    }
    for (e, parent) in parents {
        let health_percent = {
            if let Ok(health) = handle.world.get::<&Health>(parent) {
                health.current / health.max
            } else {
                to_despawn.push(e);
                continue;
            }
        };
        if let Ok(health_bar) = handle.world.query_one_mut::<&mut HealthBar>(e) {
            if health_percent != health_bar.2 {
                health_bar.2 = health_percent;
                health_bar.1.reset();
            } else {
                health_bar.1.step(handle.delta_time);
            }
        }
    }

    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}