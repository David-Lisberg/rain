use std::fs::File;
use std::path::Path;

use hecs::Entity;
use rain::engine::color::Color;
use rain::engine::core::*;
use rain::engine::input::*;
use rain::engine::component::*;
use glam::*;

use crate::game::physics::*;
use crate::game::player::input::*;
use crate::game::world::chunk::construct_chunk_mesh;
use crate::game::world::chunk::generate_chunk;
use crate::game::world::generation::system_world_generation;
use crate::game::world::tile::read_tile;
use crate::game::world::tile::write_tile;

pub mod game {
    pub mod world {
        pub mod generation;
        pub mod chunk;
        pub mod tile;
    }
    pub mod physics;
    pub mod player {
        pub mod input;
    }
}

struct State;
struct Player;

fn system_player_walk(handle: &mut RainHandle) {
    for (_, (_, _, velocity, direction)) in handle.world.query::<(
        &Player, &Walk, &mut Velocity2D, &Direction
    )>().iter() {
        set_velocity_clamped(velocity, 5.0, direction);
    }
}

// fn system_lifetime(handle: &mut RainHandle) {
//     for (_, _) in handle.world.query::<&mut Lifetime>().iter() {
        
//     }
// }

fn system_player_dash(handle: &mut RainHandle) {
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

fn system_camera_controller(handle: &mut RainHandle) {
    if handle.is_key_pressed(KeyboardKey::ArrowUp) {
        handle.renderer.camera.add_xy(0.0, 0.1);
    }
    if handle.is_key_pressed(KeyboardKey::ArrowDown) {
        handle.renderer.camera.add_xy(0.0, -0.1);
    }
    if handle.is_key_pressed(KeyboardKey::ArrowLeft) {
        handle.renderer.camera.add_xy(-0.1, 0.0);
    }
    if handle.is_key_pressed(KeyboardKey::ArrowRight) {
        handle.renderer.camera.add_xy(0.1, 0.0);
    }
    if handle.is_key_pressed(KeyboardKey::Z) {
        handle.renderer.camera.add_z(-0.15);
    }
    if handle.is_key_pressed(KeyboardKey::X) {
        handle.renderer.camera.add_z(0.15);
    }
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        system_world_generation(handle);
        system_physics_friction(handle);
        system_player_input(handle);
        system_player_walk(handle);
        system_player_dash(handle);
        system_physics_movement_2d(handle);
        system_camera_controller(handle);
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(100, 100, 100, 255));
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.load_texture("circle", "res/texture/white_circle.png").expect("Error loading texture.");
        handle.load_texture("tile_dirt", "res/texture/dirt.png").expect("Error loading texture.");
        handle.load_texture("tile_grass", "res/texture/grass.png").expect("Error loading texture.");
        handle.load_texture("tile_stone", "res/texture/stone.png").expect("Error loading texture.");
        handle.world.spawn((
            Player, Sprite, Visible, 
            Position2D{ x: 0.0, y: 0.0}, Velocity2D{ x: 0.0, y: 0.0 }, Acceleration2D{ x: 0.0, y: 0.0 }, Friction(25.0),
            Scale2D(Vec2::new(0.8, 0.8)), Direction::Down, Color::LIME, Priority(0),
        ));
        handle.renderer.camera.set_z(8.0);
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
