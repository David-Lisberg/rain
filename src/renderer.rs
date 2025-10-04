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

#[derive(Debug)]
struct BufferSegment {
    id: u32,
    vertices: Vec<UIVertex>,
    vertices_start: u32,
    vertices_length: u32,
    indices: Vec<u16>,
    indices_start: u32,
    indices_length: u32,
}

impl BufferSegment {
    fn new(id: u32) -> Self {
        Self {
            id,
            vertices: Vec::new(),
            vertices_start: 0,
            vertices_length: 0,
            indices: Vec::new(),
            indices_start: 0,
            indices_length: 0,
        }
    }
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let mut buffer_segments: [BufferSegment; 2] = [
            BufferSegment::new(ARRAY_256X256_ID),
            BufferSegment::new(ARRAY_4096X4096_ID)
        ];

        let vertex_stride = std::mem::size_of::<UIVertex>()as u32;
        let index_stride = std::mem::size_of::<u16>() as u32;
        let mut vertex_offset = 0;
        let mut index_offset = 0;
        for buffer_segment in &mut buffer_segments {
            buffer_segment.vertices_start = vertex_offset * vertex_stride;
            buffer_segment.indices_start = index_offset * index_stride;
            for draw_call in &self.draw_pass.draw_calls {
                if let DrawCall::Mesh(mesh) = draw_call {
                    if mesh.material.array_id == buffer_segment.id {
                        buffer_segment.vertices.extend(&mesh.vertices);
                        buffer_segment.indices.extend(mesh.indices.iter().map(|i| i + vertex_offset as u16));
                        vertex_offset += mesh.vertices.len() as u32;
                        index_offset += mesh.indices.len() as u32;
                    }
                }
            }
            buffer_segment.vertices_length = vertex_offset * vertex_stride - buffer_segment.vertices_start;
            buffer_segment.indices_length = index_offset * index_stride - buffer_segment.indices_start;
            // dbg!(buffer_segment);
        }

        let ui_vertex_buffer = &self.ui_vertex_buffers[self.ui_current_frame];
        let ui_index_buffer = &self.ui_index_buffers[self.ui_current_frame];

        self.ui_current_frame ^= 1;

        for buffer_segment in &buffer_segments {
            self.queue.write_buffer(ui_vertex_buffer, buffer_segment.vertices_start as u64, bytemuck::cast_slice(&buffer_segment.vertices));
            self.queue.write_buffer(ui_index_buffer, buffer_segment.indices_start as u64, bytemuck::cast_slice(&buffer_segment.indices));
        }

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

            for ((_, (_, bind_group)), buffer_segment) in resource_manager.texture_arrays.iter().zip(buffer_segments.iter()) {
                if buffer_segment.vertices_length != 0 {
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice((buffer_segment.vertices_start as u64)..((buffer_segment.vertices_start + buffer_segment.vertices_length) as u64)));
                    render_pass.set_index_buffer(ui_index_buffer.slice((buffer_segment.indices_start as u64)..((buffer_segment.indices_start + buffer_segment.indices_length) as u64)), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..(buffer_segment.indices_length / index_stride), 0, 0..1);
                }
            }
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