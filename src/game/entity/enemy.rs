use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::animation::Animation;
use rain::engine::texture::Texture;
use rain::engine::resource::ResourceManager;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rand::RngExt;

use crate::game::core::animation::AnimationStateUpdated;
use crate::game::entity::damage::{Health, HealthBar, HurtBox};
use crate::game::utility::timer::Timer;
use crate::game::world::water::Swimmable;
use crate::{DEPTH_PLAYER, State};
use crate::game::core::collision::{Collider, check_collision_with_object};
use crate::game::entity::ai::Idle;
use crate::game::entity::loot::LootTable;
use crate::game::player::item::{Item, ItemType};
use crate::game::player::movement::Player;
use crate::game::utility::direction::Direction4;

const SPAWN_RADIUS_MIN: f32 = 20.0;
const SPAWN_RADIUS_MAX: f32 = 40.0;
const DESPAWN_RADIUS: f32 = 50.0;
const SPAWN_CAP: i32 = 5;

pub enum AnimationStateEnemy {
    None,
    Walking(Direction4),
}

pub struct Enemy {
    pub _type: EnemyType,
    pub walk_speed: f32,
    pub swim_speed: f32,
    pub attack_speed: f32,
    pub damage: f32,
    pub sight_range: f32,
    pub tracking_range: f32,
    pub tracking_distance: f32,
    pub idle_interval: i32,
}

impl Enemy {
    pub fn new(_type: EnemyType) -> Self {
        let (walk_speed, swim_speed, attack_speed, damage, sight_range, tracking_range, tracking_distance, idle_interval) = match _type {
            EnemyType::Coati => (2.0, 1.5, 10.0, 10.0, 10.0, 25.0, 3.0, 600),
            EnemyType::Squirrel(_) => (2.0, 1.5, 10.0, 10.0, 10.0, 10.0, 5.0, 200),
        };
        Self {
            _type,
            walk_speed,
            swim_speed,
            attack_speed,
            damage,
            sight_range,
            tracking_range,
            tracking_distance,
            idle_interval,
        }
    }
}

pub enum EnemyType {
    Coati,
    Squirrel(i32),
}

impl EnemyType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            EnemyType::Coati => resource_manager.fetch_texture("enemy_coati_side").unwrap(),
            EnemyType::Squirrel(_) => resource_manager.fetch_texture("enemy_squirrel_side").unwrap(),
        }
    }
}

pub fn system_manage_enemies(handle: &mut RainHandle, state: &mut State) {
    if state.counter % 300 != 0 {
        return;
    }
    let mut player_position: Option<Vec2> = None;
    let mut enemy_position: Option<Vec2> = None;
    let mut to_remove: Vec<Entity> = Vec::new();
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player_position = Some(position.0.clone());
    }
    if let Some(position) = player_position {
        for (e, (_, enemy_position)) in handle.world.query::<(&Enemy, &Position2D)>().iter() {
            let distance = (enemy_position.0 - position).length();
            if distance > DESPAWN_RADIUS {
                to_remove.push(e);
            }
        }
        if state.enemy_count < SPAWN_CAP {
            let radius = state.rng.random::<f32>() * (SPAWN_RADIUS_MAX - SPAWN_RADIUS_MIN) + SPAWN_RADIUS_MIN;
            let angle = state.rng.random::<f32>() * 2.0 * std::f32::consts::PI;
            let x = f32::cos(angle) * radius + position.x;
            let y = f32::sin(angle) * radius + position.y;
            enemy_position = Some(Vec2::new(x, y));
        }
    }
    if let Some(position) = enemy_position {
        spawn_enemy(handle, state, position, Enemy::new(EnemyType::Squirrel(0)));
    }
    for e in to_remove {
        state.enemy_count -= 1;
        handle.world.despawn(e).unwrap();
    }
}

pub fn spawn_enemy(handle: &mut RainHandle, state: &mut State, position: Vec2, enemy: Enemy) {
    let texture = enemy._type.fetch_texture(&handle.resource_manager);
    let (loot_table, scale) = match enemy._type {
        EnemyType::Coati => {
            ( LootTable { drops: vec![
                (1.0, 1..=3, Item::new(ItemType::CoatiPelt)),
                (1.0, 1..=3, Item::new(ItemType::SmallBone)),
                (0.5, 1..=1, Item::new(ItemType::BonePlate))
            ] },
            Scale2D(Vec2::new(1.0, 1.0)) )
        }
        EnemyType::Squirrel(_) => {
            ( LootTable { drops: vec![
                (1.0, 1..=3, Item::new(ItemType::SquirrelPelt)),
                (0.5, 1..=2, Item::new(ItemType::SmallBone))
            ] },
            Scale2D(Vec2::new(0.6, 0.6)) )
        }
    };
    let collider = Collider::from_center(position.x, position.y, scale.0.x * 0.8, scale.0.y * 0.8);

    if check_collision_with_object(state, &collider).is_some() {
        return;
    }
    state.enemy_count += 1;

    let e = handle.world.spawn((Sprite, Visible, enemy, Idle, Position2D(position), Velocity2D(Vec2::ZERO), Acceleration2D(Vec2::ZERO), 
        texture, scale, DepthZ(DEPTH_PLAYER), Priority(1), Flip(false, false), Health::new(5.0), collider, HurtBox(collider)));
    handle.world.insert(e, (Swimmable, loot_table, AnimationStateEnemy::None)).unwrap();

    handle.world.spawn((HealthBar(e, Timer::new(2.0), 1.0),));
}

pub fn system_update_enemy_direction(handle: &mut RainHandle) {
    let mut to_add_direction: Vec<(Entity, Direction)> = Vec::new();

    for (e, (_, velocity, direction)) in handle.world.query_mut::<(&Enemy, &Velocity2D, Option<&mut Direction>)>() {
        let new_direction = velocity.0.normalize();
        if let Some(d) = direction {
            if velocity.0 != Vec2::ZERO {
                *d = Direction(new_direction);
            }
        } else {
            to_add_direction.push((e, Direction(new_direction)));
        }
    }

    for (e, direction) in to_add_direction {
        handle.world.insert_one(e, direction).unwrap();
    }
}

pub fn system_update_enemy_texture(handle: &mut RainHandle) {
    let mut to_add_texture: Vec<(Entity, String)> = Vec::new();

    for (e, (enemy, direction, flip)) in handle.world.query_mut::<(&Enemy, &Direction, &mut Flip)>() {
        let direction4 = Direction4::from_vec2(direction.0);
        match direction4 {
            Direction4::N => {
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_back".to_string())),
                    EnemyType::Squirrel(_) => to_add_texture.push((e, "enemy_squirrel_back".to_string())),
                }
            }
            Direction4::E => {
                *flip = Flip(false, false);
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_side".to_string())),
                    EnemyType::Squirrel(_) => to_add_texture.push((e, "enemy_squirrel_side".to_string())),
                }
            }
            Direction4::S => {
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_front".to_string())),
                    EnemyType::Squirrel(_) => to_add_texture.push((e, "enemy_squirrel_front".to_string())),
                }
            }
            Direction4::W => {
                *flip = Flip(true, false);
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_side".to_string())),
                    EnemyType::Squirrel(_) => to_add_texture.push((e, "enemy_squirrel_side".to_string())),
                }
            }
        }
    }

    for (e, id) in to_add_texture {
        let new_texture = handle.fetch_texture(&id).unwrap();
        if let Ok(texture) = handle.world.query_one_mut::<&mut Arc<Texture>>(e) {
            *texture = new_texture;
        }
    }
}

pub fn system_update_enemy_animation(handle: &mut RainHandle) {
    let mut to_add_animation: Vec<(Entity, Animation)> = Vec::new();
    let mut to_remove_updated: Vec<Entity> = Vec::new();
    let mut to_remove_animation: Vec<Entity> = Vec::new();

    for (e, (enemy, state, _)) in handle.world.query::<(&Enemy, &AnimationStateEnemy, &AnimationStateUpdated)>().iter() {
        to_remove_updated.push(e);
        match state {
            AnimationStateEnemy::None => to_remove_animation.push(e),
            AnimationStateEnemy::Walking(direction) => {
                let animation_string: Option<String> = match enemy._type {
                    EnemyType::Coati => Some("animation_coati_walking_".to_string()),
                    _ => None,
                };
                let end_string = match direction {
                    Direction4::N => "back",
                    Direction4::S => "front",
                    Direction4::E | Direction4::W => "side",
                };
                if let Some(mut string) = animation_string {
                    string.push_str(end_string);
                    to_add_animation.push((e, Animation::new(&string)));
                }
            }
        }
    }

    for (e, animation) in to_add_animation {
        handle.world.insert_one(e, animation).unwrap();
    }

    for e in to_remove_updated {
        handle.world.remove_one::<AnimationStateUpdated>(e).unwrap();
    }
    for e in to_remove_animation {
        let _ = handle.world.remove_one::<Animation>(e).is_ok();
    }
}