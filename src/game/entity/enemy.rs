use std::collections::HashMap;
use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::animation::Animation;
use rain::engine::texture::Texture;
use rain::engine::resource::ResourceManager;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rand::RngExt;
use serde::Deserialize;

use crate::game::core::animation::AnimationStateUpdated;
use crate::game::entity::damage::{Health, HealthBar, HurtBox};
use crate::game::entity::transition::TransitionGraph;
use crate::game::utility::timer::Timer;
use crate::game::world::water::Swimmable;
use crate::{DEPTH_PLAYER, State};
use crate::game::core::collision::{Collider, check_collision_with_object};
use crate::game::entity::ai::Idle;
use crate::game::entity::loot::LootTable;
use crate::game::player::item::{Item, ItemType};
use crate::game::player::movement::Player;
use crate::game::utility::direction::Direction4;

const SPAWN_RADIUS_MIN: f32 = 40.0;
const SPAWN_RADIUS_MAX: f32 = 60.0;
const DESPAWN_RADIUS: f32 = 65.0;
const SPAWN_CAP: i32 = 1;

pub enum AnimationStateEnemy {
    None,
    Walking(Direction4),
}

pub type EnemyRegistry = HashMap<EnemyType, EnemyData>;

#[derive(Deserialize, Clone)]
pub struct EnemyData {
    pub transition_graph: TransitionGraph,
    pub health: f32,
    pub texture_side: String,
    pub texture_front: Option<String>,
    pub texture_back: Option<String>,
    pub resource: Option<i32>,
    pub max_resource: Option<i32>,
    pub walk_speed: f32,
    pub swim_speed: Option<f32>,
    pub attack_speed: f32,
    pub damage: f32,
    pub sight_range: f32,
    pub tracking_range: f32,
    pub tracking_distance: f32,
    pub idle_interval: i32,
}

pub struct Enemy(pub EnemyType);

pub struct Resource {
    pub current: i32,
    pub max: Option<i32>,
}
pub struct Diggable;

// EnemyType::Deer => (1.0, 0.5, 10.0, 15.0, 8.0, 12.0, 12.0, 600),

#[derive(Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EnemyType {
    Coati,
    Squirrel,
    // Deer,
}

impl EnemyType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            EnemyType::Coati => resource_manager.fetch_texture("enemy_coati_side").unwrap(),
            EnemyType::Squirrel => resource_manager.fetch_texture("enemy_squirrel_side").unwrap(),
            // EnemyType::Deer => resource_manager.fetch_texture("enemy_deer_side").unwrap(),
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
        spawn_enemy(handle, state, position, Enemy(EnemyType::Squirrel));
    }
    for e in to_remove {
        state.enemy_count -= 1;
        handle.world.despawn(e).unwrap();
    }
}

pub fn spawn_enemy(handle: &mut RainHandle, state: &mut State, position: Vec2, enemy: Enemy) {
    let enemy_data = state.enemy_registry.get(&enemy.0).unwrap().clone();
    let texture = handle.fetch_texture(&enemy_data.texture_side).unwrap();
    let (loot_table, scale) = match enemy.0 {
        EnemyType::Coati => {
            ( LootTable { drops: vec![
                (1.0, 1..=3, Item::new(ItemType::CoatiPelt)),
                (1.0, 1..=3, Item::new(ItemType::SmallBone)),
                (0.5, 1..=1, Item::new(ItemType::BonePlate))
            ] },
            Scale2D(Vec2::new(1.0, 1.0)) )
        }
        EnemyType::Squirrel => {
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

    if let Some(current) = enemy_data.resource {
        let resource = Resource { current, max: enemy_data.max_resource };
        handle.world.insert_one(e, resource).unwrap();
    }

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

pub fn system_update_enemy_texture(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_texture: Vec<(Entity, String)> = Vec::new();

    for (e, (enemy, direction, flip)) in handle.world.query_mut::<(&Enemy, &Direction, &mut Flip)>() {
        let direction4 = Direction4::from_vec2(direction.0);
        let enemy_data = state.enemy_registry.get(&enemy.0).unwrap();
        match direction4 {
            Direction4::N => {
                match enemy_data.texture_back.clone() {
                    Some(t) => to_add_texture.push((e, t)),
                    None => to_add_texture.push((e, enemy_data.texture_side.clone())),
                }
            }
            Direction4::E => {
                *flip = Flip(false, false);
                to_add_texture.push((e, enemy_data.texture_side.clone()));
            }
            Direction4::S => {
                match enemy_data.texture_front.clone() {
                    Some(t) => to_add_texture.push((e, t)),
                    None => to_add_texture.push((e, enemy_data.texture_side.clone())),
                }
            }
            Direction4::W => {
                *flip = Flip(true, false);
                to_add_texture.push((e, enemy_data.texture_side.clone()));
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
                let animation_string: Option<String> = match enemy.0 {
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