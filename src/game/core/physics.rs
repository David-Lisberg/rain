use hecs::Entity;
use rain::engine::core::RainHandle;
use rain::engine::component::*;

use crate::State;
use crate::game::core::collision::Collider;
use crate::game::entity::damage::{HitBox, HurtBox};
use crate::game::world::chunk::{ChunkPosition, position_to_chunk_position};

pub const ADJACENT: [(i32, i32); 9] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 0), (0, 1), (1, -1), (1, 0), (1, 1)];

pub fn system_physics_movement_2d(handle: &mut RainHandle, state: &mut State) {
    let mut colliders: Vec<(Option<Entity>, Collider)> = Vec::new();
    for (e, collider) in handle.world.query::<&Collider>().iter() {
        colliders.push((Some(e), collider.clone()));
    }

    for (e, (position, velocity, acceleration, collider, hitbox, hurtbox)) in handle.world.query::<(
        &mut Position2D, &mut Velocity2D, &Acceleration2D, Option<&mut Collider>, Option<&mut HitBox>, Option<&mut HurtBox>
    )>().iter() {
        velocity.0 += acceleration.0 * handle.delta_time;

        let mut position_delta = velocity.0 * handle.delta_time;
        if let Some(c) = collider {
            let new_collider = Collider::new(c.x + position_delta.x, c.y + position_delta.y, c.width, c.height);
            let chunk_position: ChunkPosition = position_to_chunk_position(new_collider.x, new_collider.y);
            let mut object_colliders: Vec<Collider> = Vec::new();

            for adjacent in ADJACENT {
                let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
                if let Some(chunk) = state.chunks.get(&adjacent_position) {
                    for object in &chunk.objects {
                        if object.collidable {
                            object_colliders.push(object.collider.clone());
                        }
                    }
                }
            }

            let object_colliders: Vec<(Option<Entity>, Collider)> = object_colliders.iter()
                .map(|c| (None, c.clone()))
                .collect();

            for (entity, other_collider) in colliders.iter().chain(object_colliders.iter()) {
                if let Some(other_e) = entity {
                    if e == *other_e {
                        continue;
                    } 
                }

                if new_collider.aabb_collision(other_collider) {
                    let overlap_x = (new_collider.x + new_collider.width / 2.0) - (other_collider.x + other_collider.width / 2.0);
                    let overlap_y = (new_collider.y + new_collider.height / 2.0) - (other_collider.y + other_collider.height / 2.0);
    
                    let penetration_x = (new_collider.width + other_collider.width) / 2.0 - overlap_x.abs();
                    let penetration_y = (new_collider.height + other_collider.height) / 2.0 - overlap_y.abs();
                    
                    if penetration_x < penetration_y {
                        position_delta.x += penetration_x * overlap_x.signum();
                        velocity.0.x = 0.0;
                    } else {
                        position_delta.y += penetration_y * overlap_y.signum();
                        velocity.0.y = 0.0;
                    }
                }
            }

            c.x += position_delta.x;
            c.y += position_delta.y;
        }
        if let Some(h) = hitbox {
            h.collider.x += position_delta.x;
            h.collider.y += position_delta.y;
        }
        if let Some(h) = hurtbox {
            h.0.x += position_delta.x;
            h.0.y += position_delta.y;
        }

        position.0 += position_delta;
    }
}

pub fn system_physics_friction(handle: &mut RainHandle) {
    for (_, (velocity, acceleration, friction)) in handle.world.query::<(
        &mut Velocity2D, &mut Acceleration2D, &Friction
    )>().iter() {
        if velocity.0.x > 0.1 {
            acceleration.0.x = -friction.0;
        } else if velocity.0.x < -0.1 {
            acceleration.0.x = friction.0;
        } else {
            acceleration.0.x = 0.0;
            velocity.0.x = 0.0;
        }
        if velocity.0.y > 0.1 {
            acceleration.0.y = -friction.0;
        } else if velocity.0.y < -0.1 {
            acceleration.0.y = friction.0;
        } else {
            acceleration.0.y = 0.0;
            velocity.0.y = 0.0;
        }
    }
}

pub fn set_velocity_clamped(velocity: &mut Velocity2D, magnitude: f32, direction: &Direction) {
    let new_velocity = direction.0 * magnitude;

    if new_velocity.x > 0.01 {
        velocity.0.x = new_velocity.x.max(velocity.0.x);
    } else if new_velocity.x < -0.01 {
        velocity.0.x = new_velocity.x.min(velocity.0.x);
    }
    if new_velocity.y > 0.01 {
        velocity.0.y = new_velocity.y.max(velocity.0.y);
    } else if new_velocity.y < -0.01 {
        velocity.0.y = new_velocity.y.min(velocity.0.y);
    }
}