use rain::{color::Color, core::*};
use winit::event_loop::EventLoop;

struct State;

impl RainState for State {
    fn update(&mut self, _handle: &RainHandle) {
        
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::RED);
        handle.draw_rectangle(200.0, 200.0, 100.0, 300.0, Color::GREEN);
    }
}

fn main() -> anyhow::Result<()> {
    let state = State;
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = RainApp::new(state);
    event_loop.run_app(&mut app)?;

    Ok(())
}
