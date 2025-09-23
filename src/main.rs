use reign::core::*;
use winit::event_loop::EventLoop;

struct State;

impl ReignState for State {
    fn update(&mut self, handle: &ReignHandle) {
        
    }

    fn render(&mut self, handle: &mut ReignHandle) {
        
    }
}

fn main() -> anyhow::Result<()> {
    let state = State;
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = ReignApp::new(state);
    event_loop.run_app(&mut app)?;

    Ok(())
}
