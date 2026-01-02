use hecs::Entity;
use hecs::World;
use rain::engine::color::Color;
use rain::engine::core::*;
use rain::engine::input::*;
use rain::engine::component::*;
use glam::*;

pub mod game {
    pub mod world;
}

struct State;
struct Player;

fn system_physics_movement_2d(handle: &mut RainHandle) {
    for (_, (position, velocity, acceleration)) in handle.world.query::<(
        &mut Position2D, &mut Velocity2D, &Acceleration2D
    )>().iter() {
        velocity.x += acceleration.x * handle.delta_time;
        velocity.y += acceleration.y * handle.delta_time;
        position.x += velocity.x * handle.delta_time;
        position.y += velocity.y * handle.delta_time;
    }
}

fn system_player_walk(handle: &mut RainHandle) {
    for (_, (_, _, velocity, direction)) in handle.world.query::<(
        &Player, &Walk, &mut Velocity2D, &Direction
    )>().iter() {
        set_velocity_clamped(velocity, 2.0, direction);
    }
}

fn system_friction(handle: &mut RainHandle) {
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

fn system_lifetime(handle: &mut RainHandle) {
    for (_, _) in handle.world.query::<&mut Lifetime>().iter() {
        
    }
}

fn system_player_dash(handle: &mut RainHandle) {
    let mut entities: Vec<Entity> = Vec::new();
    for (e, (_, _, direction, velocity)) in handle.world.query::<(
        &Player, &Dash, &Direction, &mut Velocity2D
    )>().iter() {
        entities.push(e);
        set_velocity_clamped(velocity, 6.0, direction);
    }

    for e in entities {
        let _ = handle.world.remove_one::<Dash>(e);
    }
}

fn set_velocity_clamped(velocity: &mut Velocity2D, magnitude: f32, direction: &Direction) {
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

fn system_player_input(handle: &mut RainHandle) {
    let mut to_dash: Vec<Entity> = Vec::new();
    let mut to_walk: Vec<Entity> = Vec::new();
    let mut to_remove_walk: Vec<Entity> = Vec::new();
    for (e, (_, direction)) in handle.world.query::<(&Player, &mut Direction)>().iter() {
        if handle.is_key_pressed(KeyboardKey::A) && handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction::UpLeft;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) && handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction::UpRight;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) && handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction::DownRight;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) && handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction::DownLeft;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::A) {
            *direction = Direction::Left;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::D) {
            *direction = Direction::Right;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::S) {
            *direction = Direction::Down;
            to_walk.push(e);
        } else if handle.is_key_pressed(KeyboardKey::W) {
            *direction = Direction::Up;
            to_walk.push(e);
        } else {
            to_remove_walk.push(e);
        }
        if handle.is_key_released(KeyboardKey::Space) {
            to_dash.push(e);
        }
    }

    for e in to_dash {
        handle.world.insert_one(e, Dash).unwrap();
    }
    for e in to_walk {
        handle.world.insert_one(e, Walk).unwrap();
    }
    for e in to_remove_walk {
        let _ = handle.world.remove_one::<Walk>(e);
    }
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        system_friction(handle);
        system_player_input(handle);
        system_player_walk(handle);
        system_player_dash(handle);
        system_physics_movement_2d(handle);
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(100, 100, 100, 255));
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.load_texture("circle", "res/texture/white_circle.png").expect("Error loading texture.");
        handle.world.spawn((
            Player, Sprite, Visible, 
            Position2D{ x: 0.0, y: 0.0}, Velocity2D{ x: 0.0, y: 0.0 }, Acceleration2D{ x: 0.0, y: 0.0 }, Friction(10.0),
            Scale2D(Vec2::new(0.2, 0.2)), Direction::Down, Color::LIME
        ));
        handle.world.spawn_batch((0..100).map(|i| (
            Sprite, Visible, Position2D{ x: 5.0 - (i / 10) as f32, y: 5.0 - (i % 10) as f32 }, Scale2D(Vec2::new(1.0, 1.0)),
            if (i % 10 + i / 10) % 2 == 0 {
                Color::WHITE
            } else {
                Color::BLACK
            }
        )));
    }
}

fn main() -> anyhow::Result<()> {
    let state = State;
    let _ = RainApp::new(state)
        .size(850, 600)
        .title("hello_world")
        .run();

    Ok(())
}
