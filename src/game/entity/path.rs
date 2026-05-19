use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::{Position2D, Velocity2D}, core::RainHandle};

const BASE_VELOCITY: f32 = 2.0;

pub struct Path {
    positions: Vec<Vec2>,
    current: usize,
}

impl Path {
    pub fn new(positions: Vec<Vec2>) -> Self {
        Self { positions, current: 0 }
    }
}

pub fn system_path_walk(handle: &mut RainHandle) {
    let mut to_remove_path: Vec<Entity> = Vec::new();

    for (e, (position, velocity, path)) in handle.world.query_mut::<(&Position2D, &mut Velocity2D, &mut Path)>() {
        let (current, _) = path.positions.iter()
            .enumerate()
            .min_by(|a, b| (a.1 - position.0).length().partial_cmp(&(b.1 - position.0).length()).unwrap())
            .unwrap();
        if let Some(next) = path.positions.get(current + 1) {
            let direction = (*next - path.positions[current]).normalize();
            velocity.0 = BASE_VELOCITY * direction;
        } else {
            to_remove_path.push(e);
            velocity.0 = Vec2::ZERO;
        }
        path.current = current;
    }

    for e in to_remove_path {
        handle.world.remove_one::<Path>(e).unwrap();
    }
}