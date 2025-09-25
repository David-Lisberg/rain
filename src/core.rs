use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::renderer::Renderer;

use super::input::*;

pub trait ReignState {
    fn update(&mut self, handle: &ReignHandle);
    fn render(&mut self, handle: &mut ReignHandle);
}

pub struct ReignHandle {
    pub renderer: Renderer,
    window: Arc<Window>,
    keyboard: Keyboard,
    mouse: Mouse,
    last_time: Instant,
    delta_time: Duration,
}

impl ReignHandle {
    async fn new(window: Arc<Window>) -> anyhow::Result<ReignHandle> {
        let renderer = Renderer::new(Arc::clone(&window)).await?;

        Ok(Self {
            renderer,
            window,
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            last_time: Instant::now(),
            delta_time: Duration::ZERO,
        })
    }

    fn _update(&mut self) {

    }

    fn handle_input_keyboard(&mut self, code: KeyCode, is_pressed: bool) {
        let keyboard_key = KeyboardKey::from(code);
        let key = &mut self.keyboard.keys[keyboard_key as usize];
        if is_pressed {
            key.pressed = true;
            key.just_pressed = true;
        } else {
            key.pressed = false;
            key.released = true;
        }
    }

    fn handle_input_mouse(&mut self, code: winit::event::MouseButton, is_pressed: bool) {
        let mouse_button = MouseButton::from(code);
        let button = &mut self.mouse.buttons[mouse_button as usize];
        if is_pressed {
            button.pressed = true;
            button.just_pressed = true;
        } else {
            button.pressed = false;
            button.released = true;
        }
    }
}

pub struct ReignApp<F> 
where 
    F: ReignState + 'static
{
    handle: Option<ReignHandle>,
    state: Option<F>,
}

impl<F> ReignApp<F>
where
    F: ReignState + 'static,
{
    pub fn new(state: F) -> Self {
        Self {
            handle: None,
            state: Some(state),
        }
    }
}

impl<F> ApplicationHandler<ReignHandle> for ReignApp<F>
where 
    F: ReignState + 'static
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.handle = Some(pollster::block_on(ReignHandle::new(window)).unwrap());
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ReignHandle) {
        self.handle = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let handle = match &mut self.handle {
            Some(h) => h,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => handle.renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Some(s) = &mut self.state {
                    s.render(handle);
                }
                handle.window.request_redraw();
                match handle.renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = handle.window.inner_size();
                        handle.renderer.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                handle.handle_input_mouse(button, is_pressed);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => handle.handle_input_keyboard(code, key_state.is_pressed()),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let handle = match &mut self.handle {
            Some(h) => h,
            None => return,
        };

        let now = Instant::now();
        handle.delta_time = now - handle.last_time;
        handle.last_time = now;

        if let Some(s) = &mut self.state {
            s.update(handle);
        }
    }
}