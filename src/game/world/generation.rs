use glam::Vec2;
use rain::engine::color::Color;
use rain::engine::core::RainHandle;
use rain::engine::component::*;

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