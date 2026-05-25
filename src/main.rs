use std::collections::HashMap;

use lgui::manager::GUI;
use noise::Perlin;
use rain::engine::animation::AnimationPool;
use rain::engine::color::Color;
use rain::engine::core::*;
use rain::engine::component::*;
use glam::*;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::game::core::animation::system_manage_animation_events;
use crate::game::core::camera::*;
use crate::game::core::collision::Collider;
use crate::game::core::physics::*;
use crate::game::core::ui::render_ui;
use crate::game::entity::ai::system_enemy_attacking;
use crate::game::entity::ai::system_enemy_idle;
use crate::game::entity::ai::system_enemy_tracking;
use crate::game::entity::damage::Health;
use crate::game::entity::damage::HurtBox;
use crate::game::entity::damage::system_health_bar;
use crate::game::entity::damage::system_hitbox_hurtbox_collision;
use crate::game::entity::enemy::Enemy;
use crate::game::entity::enemy::EnemyType;
use crate::game::entity::enemy::spawn_enemy;
use crate::game::entity::enemy::system_manage_enemies;
use crate::game::entity::despawn::system_timer_despawn;
use crate::game::entity::enemy::system_update_enemy_direction;
use crate::game::entity::enemy::system_update_enemy_texture;
use crate::game::entity::path::system_path_walk;
use crate::game::entity::projectile::system_manage_projectiles;
use crate::game::player::action::AnimationStatePlayer;
use crate::game::player::action::system_clear_animation_state;
use crate::game::player::action::system_player_action;
use crate::game::player::action::system_update_player_animation;
use crate::game::player::action::system_update_player_texture;
use crate::game::player::input::*;
use crate::game::player::inventory::Inventory;
use crate::game::player::inventory::system_inventory_interface;
use crate::game::player::item::Item;
use crate::game::player::item::ItemType;
use crate::game::player::item::system_item_drop_pickup;
use crate::game::player::item::system_timer_pickup;
use crate::game::player::movement::Player;
use crate::game::player::movement::system_player_dash;
use crate::game::player::movement::system_player_walk;
use crate::game::core::load::load_animations;
use crate::game::core::load::load_textures;
use crate::game::world::chunk::ChunkData;
use crate::game::world::chunk::ChunkPosition;
use crate::game::world::chunk::system_manage_chunks;
use crate::game::world::config::WorldGenConfig;
use crate::game::world::generation::system_world_generation;
use crate::game::world::object::system_object_transparency;
use crate::game::world::reset::Persistent;
use crate::game::world::reset::system_reset_world;
use crate::game::world::water::Swimmable;
use crate::game::world::water::system_swimming;

pub const SCREEN_WIDTH: f32 = 850.0;
pub const SCREEN_HEIGHT: f32 = 600.0;

const DEPTH_DIFFERENCE: f32 = -0.00001;

pub const DEPTH_PLAYER: f32 = 0.01;
pub const DEPTH_TREES: f32 = DEPTH_PLAYER + DEPTH_DIFFERENCE * 2.0;
pub const DEPTH_PROJECTILE: f32 = DEPTH_PLAYER + DEPTH_DIFFERENCE;
pub const DEPTH_SMALL_OBJECT: f32 = DEPTH_PLAYER - DEPTH_DIFFERENCE * 2.0;

pub mod game {
    pub mod world {
        pub mod generation;
        pub mod chunk;
        pub mod tile;
        pub mod object;
        pub mod config;
        pub mod reset;
        pub mod water;
    }
    pub mod core {
        pub mod physics;
        pub mod camera;
        pub mod collision;
        pub mod ui;
        pub mod animation;
        pub mod load;
    }
    pub mod player {
        pub mod action;
        pub mod input;
        pub mod movement;
        pub mod item;
        pub mod inventory;
        pub mod crafting;
    }
    pub mod utility {
        pub mod noise;
        pub mod timer;
        pub mod direction;
    }
    pub mod entity {
        pub mod enemy;
        pub mod despawn;
        pub mod damage;
        pub mod ai;
        pub mod path;
        pub mod projectile;
        pub mod loot;
    }
}

pub struct State {
    chunks: HashMap<ChunkPosition, ChunkData>,
    world_gen_config: WorldGenConfig,
    transparent_object_chunks: Vec<ChunkPosition>,
    gui: GUI,
    rng: ThreadRng,
    perlin: Perlin,
    zoom: f32,
    counter: i32,
    enemy_count: i32,
    to_reset: bool,
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        system_manage_animation_events(handle);
        system_clear_animation_state(handle);
        system_player_input(handle, self);
        system_inventory_interface(handle, self);

        system_manage_chunks(handle, self);
        system_world_generation(handle, self);

        system_manage_enemies(handle, self);
        system_swimming(handle, self);
        system_enemy_idle(handle, self);
        system_enemy_tracking(handle, self);
        system_enemy_attacking(handle);
        system_path_walk(handle);
        system_update_enemy_direction(handle);
        
        system_player_walk(handle);
        system_player_dash(handle);
        system_player_action(handle);

        system_physics_friction(handle);
        system_physics_movement_2d(handle, self);

        system_hitbox_hurtbox_collision(handle, self);
        system_manage_projectiles(handle);
        system_item_drop_pickup(handle);
        system_health_bar(handle);
        system_timer_despawn(handle);
        system_timer_pickup(handle);

        system_update_player_texture(handle);
        system_update_player_animation(handle);
        system_update_enemy_texture(handle);
        system_object_transparency(handle, self);
        system_camera_controller(handle, self);
        system_camera_tracker(handle);
        system_camera_zoom(handle, self);

        system_reset_world(handle, self);

        self.counter += 1;
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(100, 100, 100, 255));
        render_ui(handle, self);
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        load_textures(handle);
        load_animations(handle);

        let player_texture = handle.fetch_texture("player_front").unwrap();
        let player_collider = Collider::from_center(0.0, 0.0, 0.8, 0.8);
        let player_entity = handle.world.spawn((
            Player, Sprite, Visible, 
            Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO), Acceleration2D(Vec2::ZERO), Friction(25.0),
            Scale2D(Vec2::new(0.8, 0.8)), Direction(Vec2::new(0.0, -1.0)), player_texture, Priority(1), DepthZ(DEPTH_PLAYER), Flip(false, false), 
            player_collider,
            Inventory::new(36),
        ));
        handle.world.insert(player_entity, (
            Health::new(100.0), HurtBox(Collider::from_center(0.0, 0.0, 0.8, 0.8)), Persistent, Swimmable, AnimationPool::new(),
            AnimationStatePlayer::None,
        )).unwrap();

        for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
            inventory.add_item(Item::new(ItemType::FlintHatchet), 1);
        }

        // spawn_enemy(handle, self, Vec2::new(5.0, 1.0), Enemy::new(EnemyType::Coati));

        handle.renderer.camera.set_z(8.0);
    }
}

fn main() -> anyhow::Result<()> {
    let mut rng = rand::rng();
    let seed = rng.next_u32();
    let perlin = Perlin::new(seed);
    let state = State {
        chunks: HashMap::new(),
        world_gen_config: WorldGenConfig::default(),
        transparent_object_chunks: Vec::new(),
        gui: GUI::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        rng,
        perlin,
        zoom: 1.0,
        counter: 0,
        enemy_count: 0,
        to_reset: false,
    };
    let _ = RainApp::new(state)
        .size(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .title("hello_world")
        .run();

    Ok(())
}
