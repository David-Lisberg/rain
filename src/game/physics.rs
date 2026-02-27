use rain::engine::core::RainHandle;
use rain::engine::component::*;

pub fn system_physics_movement_2d(handle: &mut RainHandle) {
    for (_, (position, velocity, acceleration)) in handle.world.query::<(
        &mut Position2D, &mut Velocity2D, &Acceleration2D
    )>().iter() {
        velocity.x += acceleration.x * handle.delta_time;
        velocity.y += acceleration.y * handle.delta_time;
        position.x += velocity.x * handle.delta_time;
        position.y += velocity.y * handle.delta_time;
        println!("position: {} {}", position.x, position.y);
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