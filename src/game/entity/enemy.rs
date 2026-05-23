use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::*, core::RainHandle, resource::ResourceManager, texture::Texture};
use rand::RngExt;

use crate::{DEPTH_PLAYER, State, game::{core::collision::{Collider, check_collision_with_object}, entity::{ai::Idle, damage::{Health, HealthBar, HurtBox}, loot::LootTable}, player::{item::{Item, ItemType}, movement::Player}, utility::{direction::Direction4, timer::Timer}, world::water::Swimmable}};

const SPAWN_RADIUS_MIN: f32 = 20.0;
const SPAWN_RADIUS_MAX: f32 = 40.0;
const DESPAWN_RADIUS: f32 = 50.0;
const SPAWN_CAP: i32 = 1;

pub struct Enemy {
    pub _type: EnemyType,
    pub walk_speed: f32,
    pub swim_speed: f32,
    pub attack_speed: f32,
    pub damage: f32,
    pub sight_range: f32,
    pub tracking_range: f32,
    pub tracking_distance: f32,
}

impl Enemy {
    pub fn new(_type: EnemyType) -> Self {
        let (walk_speed, swim_speed, attack_speed, damage, sight_range, tracking_range, tracking_distance) = match _type {
            EnemyType::Coati => (2.0, 1.5, 10.0, 10.0, 10.0, 25.0, 3.0)
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
        }
    }
}

pub enum EnemyType {
    Coati,
}

impl EnemyType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            EnemyType::Coati => resource_manager.fetch_texture("enemy_coati_side").unwrap(),
        }
    }
}

pub fn system_manage_enemies(handle: &mut RainHandle, state: &mut State) {
    if state.counter % 180 != 0 {
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
        spawn_enemy(handle, state, position, Enemy::new(EnemyType::Coati));
    }
    for e in to_remove {
        state.enemy_count -= 1;
        handle.world.despawn(e).unwrap();
    }
}

pub fn spawn_enemy(handle: &mut RainHandle, state: &mut State, position: Vec2, enemy: Enemy) {
    let collider = Collider::from_center(position.x, position.y, 0.8, 0.8);
    let texture = enemy._type.fetch_texture(&handle.resource_manager);
    if check_collision_with_object(state, &collider).is_some() {
        return;
    }
    state.enemy_count += 1;
    let loot_table = match enemy._type {
        EnemyType::Coati => {
            LootTable { drops: vec![
                (1.0, 1..=3, Item::new(ItemType::CoatiPelt)),
                (1.0, 1..=3, Item::new(ItemType::CoatiBone)),
                (0.5, 1..=1, Item::new(ItemType::CoatiBonePlate))
            ] }
        }
    };

    let e = handle.world.spawn((Sprite, Visible, enemy, Idle, Position2D(position), Velocity2D(Vec2::ZERO), Acceleration2D(Vec2::ZERO), 
        texture, Scale2D(Vec2::new(1.0, 1.0)), DepthZ(DEPTH_PLAYER), Priority(1), Flip(false, false), Health::new(5.0), collider, HurtBox(collider)));
    handle.world.insert(e, (Swimmable, loot_table)).unwrap();
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
                }
            }
            Direction4::E => {
                *flip = Flip(false, false);
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_side".to_string())),
                }
            }
            Direction4::S => {
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_front".to_string())),
                }
            }
            Direction4::W => {
                *flip = Flip(true, false);
                match enemy._type {
                    EnemyType::Coati => to_add_texture.push((e, "enemy_coati_side".to_string())),
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