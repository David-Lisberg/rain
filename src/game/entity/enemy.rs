use glam::Vec2;
use rain::engine::{color::Color, component::*, core::RainHandle};
use rand::RngExt;

use crate::{DEPTH_PLAYER, State, game::{core::collision::{Collider, check_collision_with_object}, entity::damage::{Health, HurtBox}, player::movement::Player}};

const SPAWN_RADIUS_MIN: f32 = 20.0;
const SPAWN_RADIUS_MAX: f32 = 40.0;

pub fn system_spawn_enemy(handle: &mut RainHandle, state: &mut State) {
    if state.counter % 60 != 0 {
        return;
    }
    let mut enemy_position: Option<Vec2> = None;
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let radius = state.rng.random::<f32>() * (SPAWN_RADIUS_MAX - SPAWN_RADIUS_MIN) + SPAWN_RADIUS_MIN;
        let angle = state.rng.random::<f32>() * 2.0 * std::f32::consts::PI;
        let x = f32::cos(angle) * radius + position.0.x;
        let y = f32::sin(angle) * radius + position.0.y;
        enemy_position = Some(Vec2::new(x, y));
    }
    if let Some(position) = enemy_position {
        spawn_enemy(handle, state, position);
    }
}

pub fn spawn_enemy(handle: &mut RainHandle, state: &mut State, position: Vec2) {
    let collider = Collider::from_center(position.x, position.y, 0.8, 0.8);
    if check_collision_with_object(state, &collider).is_some() {
        return;
    }

    handle.world.spawn((Sprite, Visible, Position2D(position), Velocity2D(Vec2::ZERO), Acceleration2D(Vec2::ZERO), 
        Color::RED, Scale2D(Vec2::new(0.8, 0.8)), DepthZ(DEPTH_PLAYER), Priority(1), Health(5.0), collider, HurtBox(collider)));
}