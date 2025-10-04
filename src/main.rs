use rain::{color::Color, core::*};
use winit::event_loop::EventLoop;

struct State;

impl RainState for State {
    fn update(&mut self, _handle: &RainHandle) {
        
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::LIME);
        handle.draw_rectangle(50.0, 50.0, 300.0, 300.0, Color::TEAL);
        handle.draw_texture(10.0, 20.0, 200.0, 400.0, handle.fetch_texture("test").unwrap(), Color::WHITE);
    }

    fn setup(&mut self, handle: &mut RainHandle) {
        handle.load_texture("test", "res/texture/black_pawn.png").expect("Error loading texture.");
    }
}

fn main() -> anyhow::Result<()> {
    let state = State;
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = RainApp::new(state);



    event_loop.run_app(&mut app)?;

    Ok(())
}
