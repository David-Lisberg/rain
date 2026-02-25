use glam::Vec2;
use noise::core::perlin::perlin_2d;
use noise::core::simplex::simplex_2d;
use noise::permutationtable::PermutationTable;
use rain::engine::color::Color;
use rain::engine::core::RainHandle;
use rain::engine::component::*;

use noise::{Fbm, NoiseFn, Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};

pub fn system_world_generation(handle: &mut RainHandle) {
    handle.world.spawn_batch((0..100).map(|i| (
        Sprite, Visible, Position2D{ x: 5.0 - (i / 10) as f32, y: 5.0 - (i % 10) as f32 }, Scale2D(Vec2::new(1.0, 1.0)),
        if (i % 10 + i / 10) % 2 == 0 {
            Color::WHITE
        } else {
            Color::BLACK
        }
    )));
}

pub fn generate_noise(handle: &mut RainHandle) {
    let perlin = Perlin::new(0);

    handle.world.spawn_batch((0..400).map(|i| { 
        let scale_factor = 0.07;
        let value = (255.0 * perlin.get([(i / 20) as f64 * scale_factor, (i % 20) as f64 * scale_factor])) as u8;
        (Sprite, Visible, Position2D{ x: 10.0 - (i / 20) as f32, y:10.0 - (i % 20) as f32 }, Scale2D(Vec2::new(1.0, 1.0)),
        Color::new(value, value, value, 255)
    )}));
}