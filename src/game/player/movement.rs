use std::sync::Arc;

use hecs::Entity;
use rain::engine::{component::*, core::RainHandle, texture::Texture};

use crate::game::core::{animation::{Animation, AnimationFrame}, physics::set_velocity_clamped};

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
    let mut to_add_animation: Vec<(Entity, Animation, Arc<Texture>)> = Vec::new();
    let mut to_remove_animation: Vec<Entity> = Vec::new();
    for (e, (_, walk, velocity, direction, animation)) in handle.world.query::<(
        &Player, Option<&Walk>, &mut Velocity2D, &Direction, Option<&Animation>
    )>().iter() {
        if walk.is_some() {
            if animation.is_none() {
                if direction.0.y > 0.8 || direction.0.y < -0.8 {
                    
                } else if direction.0.x.is_sign_positive() || direction.0.x.is_sign_negative() {
                    to_add_animation.push((
                        e,
                        Animation::new(vec![
                            AnimationFrame::new(UVRect::new(0.0, 0.0, 0.5, 0.5), 8),
                            AnimationFrame::new(UVRect::new(0.5, 0.0, 0.5, 0.5), 8),
                            AnimationFrame::new(UVRect::new(0.0, 0.5, 0.5, 0.5), 8),
                            AnimationFrame::new(UVRect::new(0.5, 0.5, 0.5, 0.5), 8),
                        ], true),
                        handle.fetch_texture("animation_player_walking_side").unwrap(),
                    ));
                }
            }
            set_velocity_clamped(velocity, 5.0, direction);
        } else if animation.is_some() {
            to_remove_animation.push(e);
        }
    }

    for (e, animation, texture) in to_add_animation {
        handle.world.insert(e, (animation, UVRect::default(), texture)).unwrap();
    }
    for e in to_remove_animation {
        handle.world.remove::<(Animation, UVRect)>(e).unwrap();
    }
}