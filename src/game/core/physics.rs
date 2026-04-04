use glam::Vec2;
use rain::engine::core::RainHandle;
use rain::engine::component::*;
use rain::engine::mesh::ModelMesh;

use crate::game::core::collision::Collider;
use crate::game::world::chunk::{ChunkData, ChunkPosition, position_to_chunk_position};

pub fn system_physics_movement_2d(handle: &mut RainHandle) {
    for (_, (position, velocity, acceleration, collider)) in handle.world.query::<(
        &mut Position2D, &mut Velocity2D, &Acceleration2D, Option<&mut Collider>,
    )>().iter() {
        velocity.x += acceleration.x * handle.delta_time;
        velocity.y += acceleration.y * handle.delta_time;

        let mut position_delta = Vec2::new(velocity.x * handle.delta_time, velocity.y * handle.delta_time);
        if let Some(c) = collider {
            let new_collider = Collider::new(c.x + position_delta.x, c.y + position_delta.y, c.width, c.height);
            let chunk_position: ChunkPosition = position_to_chunk_position(new_collider.x, new_collider.y);

            for (_, (chunk, _)) in handle.world.query::<(&ChunkData, &ModelMesh)>().iter() {
                if chunk.position.x <= chunk_position.x + 1 &&
                   chunk.position.x >= chunk_position.x - 1 &&
                   chunk.position.y <= chunk_position.y + 1 &&
                   chunk.position.y >= chunk_position.y - 1 {
                    for object in &chunk.objects {
                        if object.collidable && new_collider.aabb_collision(&object.collider) {
                            let overlap_x = (new_collider.x + new_collider.width / 2.0) - (object.collider.x + object.collider.width / 2.0);
                            let overlap_y = (new_collider.y + new_collider.height / 2.0) - (object.collider.y + object.collider.height / 2.0);

                            let penetration_x = (new_collider.width + object.collider.width) / 2.0 - overlap_x.abs();
                            let penetration_y = (new_collider.height + object.collider.height) / 2.0 - overlap_y.abs();
                            
                            if penetration_x < penetration_y {
                                position_delta.x += penetration_x * overlap_x.signum();
                                velocity.x = 0.0;
                            } else {
                                position_delta.y += penetration_y * overlap_y.signum();
                                velocity.y = 0.0;
                            }
                        }
                    }
                }
            }

            c.x += position_delta.x;
            c.y += position_delta.y;
        }

        position.x += position_delta.x;
        position.y += position_delta.y;
    }
}

pub fn system_physics_friction(handle: &mut RainHandle) {
    for (_, (velocity, acceleration, friction)) in handle.world.query::<(
        &mut Velocity2D, &mut Acceleration2D, &Friction
    )>().iter() {
        if velocity.x > 0.1 {
            acceleration.x = -friction.0;
        } else if velocity.x < -0.1 {
            acceleration.x = friction.0;
        } else {
            acceleration.x = 0.0;
            velocity.x = 0.0;
        }
        if velocity.y > 0.1 {
            acceleration.y = -friction.0;
        } else if velocity.y < -0.1 {
            acceleration.y = friction.0;
        } else {
            acceleration.y = 0.0;
            velocity.y = 0.0;
        }
    }
}

pub fn set_velocity_clamped(velocity: &mut Velocity2D, magnitude: f32, direction: &Direction) {
    let diagonal = magnitude * 0.7071;
    match direction {
        Direction::Up => velocity.y = magnitude.max(velocity.y),
        Direction::UpRight => {
            velocity.x = diagonal.max(velocity.x);
            velocity.y = diagonal.max(velocity.y);
        }
        Direction::UpLeft => {
            velocity.x = (-diagonal).min(velocity.x);
            velocity.y = diagonal.max(velocity.y);
        }
        Direction::Down => velocity.y = (-magnitude).min(velocity.y),
        Direction::DownRight => {
            velocity.x = diagonal.max(velocity.x);
            velocity.y = (-diagonal).min(velocity.y);
        }
        Direction::DownLeft => {
            velocity.x = (-diagonal).min(velocity.x);
            velocity.y = (-diagonal).min(velocity.y);
        }
        Direction::Right => velocity.x = magnitude.max(velocity.x),
        Direction::Left => velocity.x = (-magnitude).min(velocity.x),
    }
}