use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::draw::DrawPass;
use crate::color;

use super::input::*;

pub trait ReignState {
    fn update(&mut self, handle: &ReignHandle);
    fn render(&mut self, handle: &mut ReignHandle);
}

pub struct ReignHandle {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    keyboard: Keyboard,
    mouse: Mouse,
    draw_pass: DrawPass,
    last_time: Instant,
    delta_time: Duration,
}

impl ReignHandle {
    async fn new(window: Arc<Window>) -> anyhow::Result<ReignHandle> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            draw_pass: DrawPass::new(),
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            last_time: Instant::now(),
            delta_time: Duration::ZERO,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn update(&mut self) {

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view: wgpu::TextureView = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        /* handles clear background */
        let clear_background_op = if let Some(color) = &self.draw_pass.clear_background_color {
            let wgpu_color = color::Color::reign_color_to_wgpu_color(color);
            wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu_color),
                store: wgpu::StoreOp::Store,
            }
        } else {
            wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }
        };

        let base_color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: clear_background_op,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[
                    base_color_attachment
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
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

    pub fn clear_background(color: color::Color) {
        
    }
}

fn update(reign_handle: &mut ReignHandle) {

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
            WindowEvent::Resized(size) => handle.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Some(s) = &mut self.state {
                    s.render(handle);
                }
                match handle.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = handle.window.inner_size();
                        handle.resize(size.width, size.height);
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