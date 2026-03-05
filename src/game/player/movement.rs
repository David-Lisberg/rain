use hecs::Entity;
use rain::engine::{component::*, core::RainHandle};

use crate::{game::core::physics::set_velocity_clamped};

pub struct Player;

pub fn system_player_dash(handle: &mut RainHandle) {
    let mut entities: Vec<Entity> = Vec::new();
    for (e, (_, _, direction, velocity)) in handle.world.query::<(
        &Player, &Dash, &Direction, &mut Velocity2D
    )>().iter() {
        entities.push(e);
        set_velocity_clamped(velocity, 75.0, direction);
    }

    for e in entities {
        let _ = handle.world.remove_one::<Dash>(e);
    }
}

pub fn system_player_walk(handle: &mut RainHandle) {
    for (_, (_, _, velocity, direction)) in handle.world.query::<(
        &Player, &Walk, &mut Velocity2D, &Direction
    )>().iter() {
        set_velocity_clamped(velocity, 5.0, direction);
    }
}