use hecs::Entity;
use rain::engine::{animation::Animation, component::*, core::RainHandle};

use crate::game::{core::physics::set_velocity_clamped, player::action::PlayerAttacking, world::water::Swimming};

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
    let mut to_add_animation: Vec<(Entity, Animation)> = Vec::new();
    let mut to_remove_animation: Vec<Entity> = Vec::new();
    for (e, (_, walk, velocity, direction, animation, swimming)) in handle.world.query::<(
        &Player, Option<&Walk>, &mut Velocity2D, &Direction, Option<&Animation>, Option<&Swimming>
    )>().without::<&PlayerAttacking>().iter() {
        if walk.is_some() {
            let next_animation: Option<Animation> = if direction.0.y > 0.8 {
                Some(Animation::new("animation_player_walking_back"))
            } else if direction.0.y < -0.8 {
                Some(Animation::new("animation_player_walking_front"))
            } else if direction.0.x.is_sign_positive() || direction.0.x.is_sign_negative() {
                Some(Animation::new("animation_player_walking_side"))
            } else {
                None
            };
            if let Some(next) = next_animation {
                if let Some(current) = animation {
                    if current.name != next.name {
                        to_add_animation.push((e, next));
                    }
                } else {
                    to_add_animation.push((e, next));
                }
            }
            let speed = match swimming.is_some() {
                true => 3.5,
                false => 5.0,
            };
            set_velocity_clamped(velocity, speed, direction);
        } else if animation.is_some() {
            to_remove_animation.push(e);
        }
    }

    for (e, animation) in to_add_animation {
        handle.world.insert_one(e, animation).unwrap();
    }
    for e in to_remove_animation {
        let a = handle.world.remove_one::<Animation>(e).unwrap();
    }
}