use std::collections::HashMap;

use lgui::manager::GUI;
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
use crate::game::core::ui::render_ui;
use crate::game::player::input::*;
use crate::game::player::inventory::Inventory;
use crate::game::player::item::Item;
use crate::game::player::item::ItemType;
use crate::game::player::movement::Player;
use crate::game::player::movement::system_player_dash;
use crate::game::player::movement::system_player_walk;
use crate::game::world::chunk::ChunkData;
use crate::game::world::chunk::ChunkPosition;
use crate::game::world::chunk::system_manage_chunks;
use crate::game::world::generation::system_world_generation;

pub const SCREEN_WIDTH: f32 = 850.0;
pub const SCREEN_HEIGHT: f32 = 600.0;

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
        pub mod ui;
    }
    pub mod player {
        pub mod action;
        pub mod input;
        pub mod movement;
        pub mod item;
        pub mod inventory;
    }
    pub mod utility {
        pub mod noise;
    }
}

pub struct State {
    chunks: HashMap<ChunkPosition, ChunkData>,
    gui: GUI,
    rng: ThreadRng,
    perlin: Perlin,
    zoom: f32,
    counter: i32,
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        system_manage_chunks(handle, self);
        system_world_generation(handle, self);
        system_physics_friction(handle);
        system_player_input(handle, self);
        system_player_walk(handle);
        system_player_dash(handle);
        system_physics_movement_2d(handle, self);
        system_camera_controller(handle, self);
        system_camera_tracker(handle);
        system_camera_zoom(handle, self);

        self.counter += 1;
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(100, 100, 100, 255));
        render_ui(handle, self);
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
        handle.load_texture("object_twig", "res/texture/twig.png").expect("Error loading texture.");
        handle.load_texture("inventory_slot", "res/texture/inventory_slot.png").expect("Error loading texture.");

        handle.world.spawn((
            Player, Sprite, Visible, 
            Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO), Acceleration2D(Vec2::ZERO), Friction(25.0),
            Scale2D(Vec2::new(0.8, 0.8)), Direction(Vec2::new(0.0, -1.0)), Color::LIME, Priority(1), DepthZ(0.01), 
            Collider::from_center(0.0, 0.0, 0.8, 0.8),
            Inventory::new(36),
        ));

        for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
            inventory.slots[3].item = Some(Item::new(ItemType::Twig));
            inventory.slots[3].quantity = 42;
        }
        handle.renderer.camera.set_z(8.0);
    }
}

fn main() -> anyhow::Result<()> {
    let mut rng = rand::rng();
    let seed = rng.next_u32();
    let perlin = Perlin::new(seed);
    let state = State {
        chunks: HashMap::new(),
        gui: GUI::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        rng,
        perlin,
        zoom: 1.0,
        counter: 0,
    };
    let _ = RainApp::new(state)
        .size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .title("hello_world")
        .run();

    Ok(())
}
