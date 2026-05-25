use hecs::Entity;
use rain::engine::{component::*, core::RainHandle};

use crate::game::{core::{animation::AnimationStateUpdated, physics::set_velocity_clamped}, player::action::{AnimationStatePlayer, PlayerAttacking}, utility::direction::Direction4, world::water::Swimming};

pub struct Player;

pub fn system_player_dash(handle: &mut RainHandle) {
    let mut entities: Vec<Entity> = Vec::new();
    for (e, (_, _, direction, velocity)) in handle.world.query::<(
        &Player, &Dash, &Direction, &mut Velocity2D
    )>().iter() {
        entities.push(e);
        set_velocity_clamped(velocity, 20.0, direction);
    }

    for e in entities {
        let _ = handle.world.remove_one::<Dash>(e);
    }
}

pub fn system_player_walk(handle: &mut RainHandle) {
    let mut to_add_updated: Vec<Entity> = Vec::new();
    for (e, (_, walk, velocity, direction, swimming, state)) in handle.world.query::<(
        &Player, Option<&Walk>, &mut Velocity2D, &Direction, Option<&Swimming>, &mut AnimationStatePlayer
    )>().without::<&PlayerAttacking>().iter() {
        if walk.is_some() {
            let direction4 = Direction4::from_vec2(direction.0);
            match state {
                AnimationStatePlayer::None => {
                    *state = AnimationStatePlayer::Walking(direction4);
                    to_add_updated.push(e);
                }
                AnimationStatePlayer::Walking(walk_direction) => {
                    if *walk_direction != direction4 {
                        *walk_direction = direction4;
                        to_add_updated.push(e);
                    }
                }
                _ => {}
            }
            let speed = match swimming.is_some() {
                true => 3.5,
                false => 5.0,
            };
            set_velocity_clamped(velocity, speed, direction);
        } else {
            match state {
                AnimationStatePlayer::Walking(_) => {
                    *state = AnimationStatePlayer::None;
                    to_add_updated.push(e);
                }
                _ => {}
            }
        }
    }

    for e in to_add_updated {
        handle.world.insert_one(e, AnimationStateUpdated).unwrap();
    }
}