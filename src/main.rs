use noise::Perlin;
use rain::engine::color::Color;
use rain::engine::core::*;
use rain::engine::component::*;
use glam::*;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::game::core::camera::*;
use crate::game::core::collision::Collider;
use crate::game::core::physics::*;
use crate::game::player::input::*;
use crate::game::player::movement::Player;
use crate::game::player::movement::system_player_dash;
use crate::game::player::movement::system_player_walk;
use crate::game::world::chunk::system_manage_chunks;
use crate::game::world::generation::system_world_generation;

pub mod game {
    pub mod world {
        pub mod generation;
        pub mod chunk;
        pub mod tile;
        pub mod object;
    }
    pub mod core {
        pub mod physics;
        pub mod camera;
        pub mod collision;
    }
    pub mod player {
        pub mod input;
        pub mod movement;
    }
    pub mod utility {
        pub mod noise;
    }
}

pub struct State {
    rng: ThreadRng,
    perlin: Perlin,
    zoom: f32,
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        system_manage_chunks(handle);
        system_world_generation(handle, self);
        system_physics_friction(handle);
        system_player_input(handle);
        system_player_walk(handle);
        system_player_dash(handle);
        system_physics_movement_2d(handle);
        system_camera_controller(handle, self);
        system_camera_tracker(handle);
        system_camera_zoom(handle, self);
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(100, 100, 100, 255));
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.load_texture("circle", "res/texture/white_circle.png").expect("Error loading texture.");
        handle.load_texture("tile_dirt", "res/texture/dirt.png").expect("Error loading texture.");
        handle.load_texture("tile_grass", "res/texture/grass.png").expect("Error loading texture.");
        handle.load_texture("tile_stone", "res/texture/stone.png").expect("Error loading texture.");
        handle.load_texture("tile_cobblestone", "res/texture/cobblestone.png").expect("Error loading texture.");
        handle.load_texture("tile_water", "res/texture/water.png").expect("Error loading texture.");
        handle.load_texture("tile_sand", "res/texture/sand.png").expect("Error loading texture.");
        handle.load_texture("object_tree1", "res/texture/tree1.png").expect("Error loading texture.");
        handle.world.spawn((
            Player, Sprite, Visible, 
            Position2D{ x: 0.0, y: 0.0}, Velocity2D{ x: 0.0, y: 0.0 }, Acceleration2D{ x: 0.0, y: 0.0 }, Friction(25.0),
            Scale2D(Vec2::new(0.8, 0.8)), Direction::Down, Color::LIME, Priority(0), DepthZ(0.0001), Collider::new(-0.4, -0.4, 0.8, 0.8),
        ));
        handle.renderer.camera.set_z(8.0);
    }
}

fn main() -> anyhow::Result<()> {
    let mut rng = rand::rng();
    let seed = rng.next_u32();
    let perlin = Perlin::new(seed);
    let state = State {
        rng,
        perlin,
        zoom: 1.0,
    };
    let _ = RainApp::new(state)
        .size(850, 600)
        .title("hello_world")
        .run();

    Ok(())
}
