use reign::{color::Color, core::*};
use winit::event_loop::EventLoop;

struct State {
    ticker: i32,
}

impl ReignState for State {
    fn update(&mut self, _handle: &ReignHandle) {
        self.ticker += 1;
    }

    fn render(&mut self, handle: &mut ReignHandle) {
        handle.clear_background(Color::new(self.ticker as u8, (self.ticker / 2) as u8, (self.ticker / 4) as u8, 255));
    }
}

fn main() -> anyhow::Result<()> {
    let state = State{ ticker: 0 };
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = ReignApp::new(state);
    event_loop.run_app(&mut app)?;

    Ok(())
}
