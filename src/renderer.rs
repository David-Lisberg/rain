use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{color, draw::{DrawCall, DrawPass}, include_str_root, resource::{ResourceManager, ARRAY_256X256_ID, ARRAY_4096X4096_ID}, texture::{Texture, TextureArray}, vertex::UIVertex};

const MAX_UI_BUFFER_SIZE: u64 = 0x100000;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub ui_vertex_buffers: [wgpu::Buffer; 2],
    pub ui_index_buffers: [wgpu::Buffer; 2],
    pub ui_current_frame: usize,
    pub is_surface_configured: bool,
    pub draw_pass: DrawPass,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
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
        dbg!(surface_format);
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_root!("res/shader/basic.wgsl").into()),
        });

        let texture_bind_group_layout = ResourceManager::texture_bind_group_layout(&device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &texture_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    UIVertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let ui_vertex_buffers = std::array::from_fn(|_| device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_vertex_buffer"),
            size: MAX_UI_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        let ui_index_buffers = std::array::from_fn(|_| device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui_index_buffer"),
            size: MAX_UI_BUFFER_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            ui_vertex_buffers,
            ui_index_buffers,
            ui_current_frame: 0,
            is_surface_configured: false,
            draw_pass: DrawPass::new(None),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self, resource_manager: &ResourceManager) -> Result<(), wgpu::SurfaceError> {
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
            let wgpu_color = color::Color::rain_color_to_wgpu_color(color);
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

        let mut vertices_1: Vec<UIVertex> = Vec::with_capacity(MAX_UI_BUFFER_SIZE as usize);
        let mut vertices_2: Vec<UIVertex> = Vec::with_capacity(MAX_UI_BUFFER_SIZE as usize);
        let mut indices_1: Vec<u16> = Vec::new();
        let mut indices_2: Vec<u16> = Vec::new();
        let mut offset_1 = 0;
        let mut offset_2 = 0;

        for draw_call in &self.draw_pass.draw_calls {
            match draw_call {
                DrawCall::Mesh(mesh) => {
                    match mesh.material.array_id {
                        ARRAY_256X256_ID => {
                            vertices_1.extend(&mesh.vertices);
                            indices_1.extend(mesh.indices.iter().map(|i| i + offset_1));
                            offset_1 += mesh.vertices.len() as u16;
                        }
                        ARRAY_4096X4096_ID => {
                            vertices_2.extend(&mesh.vertices);
                            indices_2.extend(mesh.indices.iter().map(|i| i + offset_2));
                            offset_2 += mesh.vertices.len() as u16;
                        }
                        _ => {}
                    }
                }
            }
        }

        let num_indices_1 = indices_1.len();
        let num_indices_2 = indices_2.len();

        let ui_vertex_buffer = &self.ui_vertex_buffers[self.ui_current_frame];
        let ui_index_buffer = &self.ui_index_buffers[self.ui_current_frame];

        self.ui_current_frame ^= 1;

        self.queue.write_buffer(ui_vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.queue.write_buffer(ui_index_buffer, 0, bytemuck::cast_slice(&indices));

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

            render_pass.set_pipeline(&self.render_pipeline);

            for (_, (_, bind_group)) in resource_manager.texture_arrays.iter() {
                render_pass.set_bind_group(0, bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
            render_pass.set_index_buffer(ui_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(num_indices as u32), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.reset_render_state();

        Ok(())
    }

    fn reset_render_state(&mut self) {
        self.draw_pass = DrawPass::new(None);
    }
}