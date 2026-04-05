use glam::Vec2;
use rain::engine::{component::*, core::RainHandle, input::KeyboardKey};

use crate::State;
use crate::game::player::movement::Player;

pub fn system_camera_controller(handle: &mut RainHandle, state: &mut State) {
    if handle.is_key_pressed(KeyboardKey::Z) {
        state.zoom *= 1.02;
    }
    if handle.is_key_pressed(KeyboardKey::X) {
        state.zoom /= 1.02;
    }
}

pub fn system_camera_tracker(handle: &mut RainHandle) {
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let camera_position = handle.renderer.camera.get_xy();
        let new_position = camera_position.lerp(Vec2::new(position.0.x, position.0.y), 0.2);

        handle.renderer.camera.set_xy(new_position.x, new_position.y);
    }
}

const ZOOM_CONSTANT: f32 = 80.0;

pub fn system_camera_zoom(handle: &mut RainHandle, state: &mut State) {
    handle.renderer.camera.set_fov(ZOOM_CONSTANT / state.zoom.powf(0.7));
}