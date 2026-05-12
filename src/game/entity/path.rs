use glam::IVec2;
use rain::engine::{component::{Position2D, Velocity2D}, core::RainHandle};

const BASE_VELOCITY: f32 = 2.0;

pub struct Path {
    positions: Vec<IVec2>,
    current: usize,
}

impl Path {
    pub fn new(positions: Vec<IVec2>) -> Self {
        Self { positions, current: 0 }
    }
}

pub fn system_path_walk(handle: &mut RainHandle) {
    for (_, (position, velocity, path)) in handle.world.query_mut::<(&Position2D, &mut Velocity2D, &mut Path)>() {
        let (current, _) = path.positions.iter()
            .enumerate()
            .min_by(|a, b| (a.1.as_vec2() - position.0).length().partial_cmp(&(b.1.as_vec2() - position.0).length()).unwrap())
            .unwrap();
        if let Some(next) = path.positions.get(current + 1) {
            let direction = (*next - path.positions[current]).as_vec2().normalize();
            *velocity = Velocity2D(BASE_VELOCITY * direction);
        }
        path.current = current;
    }
}