use glam::Vec2;
use hecs::Entity;
use rain::engine::component::*;
use rain::engine::core::RainHandle;

use crate::State;
use crate::game::core::animation::AnimationStateUpdated;
use crate::game::entity::enemy::{AnimationStateEnemy, Enemy};
use crate::game::utility::direction::Direction4;
use crate::game::world::water::Swimming;


pub struct Path {
    positions: Vec<Vec2>,
    current: usize,
}

impl Path {
    pub fn new(positions: Vec<Vec2>) -> Self {
        Self { positions, current: 0 }
    }
}

pub fn system_path_walk(handle: &mut RainHandle, state: &mut State) {
    let mut to_remove_path: Vec<Entity> = Vec::new();
    let mut to_add_updated: Vec<Entity> = Vec::new();

    for (e, (enemy, position, velocity, path, swimming, animation_state)) in handle.world.query_mut::<(
        &Enemy, &Position2D, &mut Velocity2D, &mut Path, Option<&Swimming>, &mut AnimationStateEnemy
    )>() {
        let (current, _) = path.positions.iter()
            .enumerate()
            .min_by(|a, b| (a.1 - position.0).length().partial_cmp(&(b.1 - position.0).length()).unwrap())
            .unwrap();
        if let Some(next) = path.positions.get(current + 1) {
            let direction = (*next - path.positions[current]).normalize();
            let enemy_data = state.enemy_registry.get(&enemy.0).unwrap();
            let speed = match swimming.is_some() {
                true => match enemy_data.swim_speed {
                    Some(speed) => speed,
                    None => enemy_data.walk_speed,
                }
                false => enemy_data.walk_speed,
            };
            let direction4 = Direction4::from_vec2(direction);
            match animation_state {
                AnimationStateEnemy::None => {
                    *animation_state = AnimationStateEnemy::Walking(direction4);
                    to_add_updated.push(e);
                }
                AnimationStateEnemy::Walking(walking_direction) => {
                    if direction4 != *walking_direction {
                        *walking_direction = direction4;
                        to_add_updated.push(e);
                    }
                }
            }
            velocity.0 = speed * direction;
        } else {
            *animation_state = AnimationStateEnemy::None;
            to_remove_path.push(e);
            to_add_updated.push(e);
            velocity.0 = Vec2::ZERO;
        }
        path.current = current;
    }

    for e in to_remove_path {
        handle.world.remove_one::<Path>(e).unwrap();
    }
    for e in to_add_updated {
        handle.world.insert_one(e, AnimationStateUpdated).unwrap();
    }
}