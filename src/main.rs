use rain::color::Color;
use rain::core::*;

struct State{
    rotation: f32,
}

impl RainState for State {
    fn update(&mut self, handle: &mut RainHandle) {
        self.rotation += 0.5;
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::LIME);
        handle.draw_rectangle_ex(350.0, 300.0, 300.0, 200.0, Color::TEAL, self.rotation, (0.0, 0.0));
        handle.draw_texture(10.0, 20.0, 200.0, 400.0, handle.fetch_texture("test").unwrap(), Color::WHITE);
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.load_texture("test", "res/texture/black_pawn.png").expect("Error loading texture.");
    }
}

fn main() -> anyhow::Result<()> {
    let state = State{ rotation: 0.0 };
    let _ = RainApp::new(state)
        .size(850, 600)
        .title("hello_world")
        .run();

    Ok(())
}
