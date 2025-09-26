use rain::{color::Color, core::*};
use winit::event_loop::EventLoop;

struct State {
    ticker: i32,
}

impl RainState for State {
    fn update(&mut self, _handle: &RainHandle) {
        self.ticker += 1;
    }

    fn render(&mut self, handle: &mut RainHandle) {
        handle.clear_background(Color::new(self.ticker as u8, (self.ticker / 2) as u8, (self.ticker / 4) as u8, 255));
    }
}

fn main() -> anyhow::Result<()> {
    let state = State{ ticker: 0 };
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = RainApp::new(state);
    event_loop.run_app(&mut app)?;

    Ok(())
}
