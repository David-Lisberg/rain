use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::renderer::Renderer;
use crate::resource::ResourceManager;
use crate::texture::Texture;

use super::input::*;

pub trait RainState {
    fn update(&mut self, handle: &RainHandle);
    fn render(&mut self, handle: &mut RainHandle);
    fn setup(&mut self, _handle: &mut RainHandle) {

    }
}

pub struct RainHandle {
    pub renderer: Renderer,
    pub resource_manager: ResourceManager,
    pub window: Arc<Window>,
    keyboard: Keyboard,
    mouse: Mouse,
    last_time: Instant,
    delta_time: Duration,
    pub base_width: u32,
    pub base_height: u32,
}

impl RainHandle {
    async fn new(window: Arc<Window>, base_width: u32, base_height: u32) -> anyhow::Result<RainHandle> {
        let renderer = Renderer::new(Arc::clone(&window)).await?;
        let mut resource_manager = ResourceManager::new(&renderer.device);

        let white_pixel = Texture::white_pixel();
        resource_manager.load_texture(&renderer.queue, "".to_string(), &white_pixel)?;

        Ok(Self {
            renderer,
            resource_manager,
            window,
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            last_time: Instant::now(),
            delta_time: Duration::ZERO,
            base_width,
            base_height,
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

pub struct RainApp<F> 
where 
    F: RainState + 'static
{
    handle: Option<RainHandle>,
    state: Option<F>,
    width: u32,
    height: u32,
    title: String,
}

impl<F> RainApp<F>
where
    F: RainState + 'static,
{
    pub fn new(state: F) -> Self {
        Self {
            handle: None,
            state: Some(state),
            width: 800,
            height: 600,
            title: "Rain Window".to_string(),
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::with_user_event().build()?;
        event_loop.run_app(&mut self).map_err(anyhow::Error::new)
    }
}

impl<F> ApplicationHandler<RainHandle> for RainApp<F>
where 
    F: RainState + 'static
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(PhysicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());


        self.handle = Some(pollster::block_on(RainHandle::new(window, self.width, self.height)).unwrap());
        
        let handle = self.handle.as_mut().unwrap();
        if let Some(s) = &mut self.state {
            s.setup(handle);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: RainHandle) {
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
                match handle.renderer.render(&handle.resource_manager) {
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