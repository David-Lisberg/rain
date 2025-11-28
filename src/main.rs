use hecs::World;
use rain::engine::color::Color;
use rain::engine::core::*;
use rain::engine::input::*;
use rain::engine::component::*;
use glam::*;

struct State;

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        if handle.is_key_pressed(KeyboardKey::A) {
            for (_, position) in handle.world.query::<&mut Position2D>().iter() {
                position.x -= 0.01;
            }
        }
        if handle.is_key_pressed(KeyboardKey::D) {
            for (_, position) in handle.world.query::<&mut Position2D>().iter() {
                position.x += 0.01;
            }
        }
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::LIME);
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.world.spawn((Sprite, Visible, Position2D{ x: 0.0, y: 0.0}, Color::BLACK));
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
