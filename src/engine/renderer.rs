use std::sync::Arc;

use hecs::World;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::engine::animation::{Animation, AnimationPool};
use crate::engine::camera::{Camera2d, Camera2dUniform};
use crate::engine::color::{self, Color};
use crate::engine::draw::{DrawCall, DrawPass};
use crate::engine::mesh::ModelMesh;
use crate::engine::sprite::SpriteRender;
use crate::engine::text::{TextBufferPool, TextState};
use crate::include_str_root;
use crate::engine::instance::SpriteInstance;
use crate::engine::resource::*;
use crate::engine::texture::{Texture, TextureWGPU};
use crate::engine::vertex::*;
use crate::engine::component::*;

const MAX_UI_BUFFER_SIZE: u64 = 0x100000;
const MAX_INSTANCE_BUFFER_SIZE: u64 = 0x10000;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub camera: Camera2d,
    pub camera_uniform: Camera2dUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub depth_texture: TextureWGPU,
    pub ui_pipeline: wgpu::RenderPipeline,
    pub ui_vertex_buffers: [wgpu::Buffer; 2],
    pub ui_index_buffers: [wgpu::Buffer; 2],
    pub model_pipeline: wgpu::RenderPipeline,
    pub model_vertex_buffer: wgpu::Buffer,
    pub model_index_buffer: wgpu::Buffer,
    pub sprite_pipeline: wgpu::RenderPipeline,
    pub sprite_quad_vertex_buffer: wgpu::Buffer,
    pub sprite_quad_index_buffer: wgpu::Buffer,
    pub sprite_instance_buffer: wgpu::Buffer,
    pub ui_current_frame: usize,
    pub is_surface_configured: bool,
    pub draw_pass: DrawPass,
    pub text_state: TextState,
}

#[derive(Debug)]
struct BufferSegment {
    id: u32,
    vertices: Vec<UIVertex>,
    vertices_offset: u32,
    vertices_length: u32,
    indices: Vec<u16>,
    indices_offset: u32,
    indices_length: u32,
}

#[derive(Clone)]
struct PriorityBuffer<'a> {
    meshes: Vec<&'a ModelMesh>,
    sprites: Vec<SpriteRender>,
}

impl PriorityBuffer<'_> {
    fn new() -> Self {
        Self { meshes: Vec::new(), sprites: Vec::new() }
    }
}

impl BufferSegment {
    fn new(id: u32) -> Self {
        Self {
            id,
            vertices: Vec::new(),
            vertices_offset: 0,
            vertices_length: 0,
            indices: Vec::new(),
            indices_offset: 0,
            indices_length: 0,
        }
    }
}

struct BufferSegmentSpriteInstance {
    id: u32,
    instances: Vec<SpriteInstance>,
    offset: u32,
    length: u32,
}

impl BufferSegmentSpriteInstance {
    fn new(id: u32) -> Self {
        Self {
            id,
            instances: Vec::new(),
            offset: 0,
            length: 0
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
        let present_mode = surface_caps.present_modes.iter()
            .copied()
            .find(|m| *m == wgpu::PresentMode::Fifo)
            .unwrap_or(surface_caps.present_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let camera = Camera2d::default(config.width as f32, config.height as f32);
        let mut camera_uniform = Camera2dUniform::new();
        camera_uniform.update_matrix(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout = Camera2d::camera_bind_group_layout(&device);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let depth_texture = TextureWGPU::create_depth_texture(&device, &config);

        let ui_pipeline = Self::create_pipeline_ui(&device, &config);

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

        let model_pipeline = Self::create_pipeline_model(&device, &config);

        let model_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model_vertex_buffer"),
            size: MAX_UI_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let model_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model_index_buffer"),
            size: MAX_UI_BUFFER_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sprite_pipeline = Self::create_pipeline_sprites(&device, &config);

        let sprite_quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_quad_vertex_buffer"),
            contents: bytemuck::cast_slice(&SPRITE_QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let sprite_quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_quad_index_buffer"),
            contents: bytemuck::cast_slice(&SPRITE_QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let sprite_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_instance_buffer"),
            size: MAX_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer = glyphon::TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        let mut text_buffer_pool = TextBufferPool::new();
        text_buffer_pool.set_capacity(&mut font_system, 1);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            ui_pipeline,
            ui_vertex_buffers,
            ui_index_buffers,
            model_pipeline,
            model_vertex_buffer,
            model_index_buffer,
            sprite_pipeline,
            sprite_quad_vertex_buffer,
            sprite_quad_index_buffer,
            sprite_instance_buffer,
            ui_current_frame: 0,
            camera,
            camera_buffer,
            camera_uniform,
            camera_bind_group,
            is_surface_configured: false,
            draw_pass: DrawPass::new(None),
            text_state: TextState {
                font_system,
                swash_cache,
                viewport,
                atlas,
                renderer: text_renderer,
                buffer_pool: text_buffer_pool,
                to_draw: Vec::new(),
            },
        })
    }

    fn create_pipeline_ui(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_root!("res/shader/ui.wgsl").into()),
        });

        let texture_bind_group_layout = ResourceManager::texture_bind_group_layout(&device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui_pipeline"),
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
        })
    }

    fn create_pipeline_sprites(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_root!("res/shader/sprite.wgsl").into()),
        });

        let texture_bind_group_layout = ResourceManager::texture_bind_group_layout(device);
        let camera_bind_group_layout = Camera2d::camera_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    SpriteVertex::desc(),
                    SpriteInstance::desc()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureWGPU::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
    
    fn create_pipeline_model(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str_root!("res/shader/model.wgsl").into()),
        });

        let texture_bind_group_layout = ResourceManager::texture_bind_group_layout(device);
        let camera_bind_group_layout = Camera2d::camera_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    ModelVertex::desc(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureWGPU::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;

            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.depth_texture = TextureWGPU::create_depth_texture(&self.device, &self.config);

            self.camera.aspect = width as f32 / height as f32;
            self.camera.updated = true;
        }
    }

    pub fn render(&mut self, resource_manager: &ResourceManager, world: &World) -> Result<(), wgpu::SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view: wgpu::TextureView = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

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
            resolve_target: None,
            ops: clear_background_op,
        });
        let default_color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        });

        let base_depth_ops = Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
        });
        let default_depth_ops = Some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        });

        let mut no_priority: PriorityBuffer = PriorityBuffer::new();
        let mut priority_buffer: Vec<PriorityBuffer> = Vec::new();
        let mut priority_buffer_value: Vec<i32> = Vec::new();
    
        let mut query = world.query::<(
            &Visible, Option<&Priority>, Option<&ModelMesh>, Option<&Sprite>, Option<&Animation>, Option<&AnimationPool>,
            Option<&Position2D>, Option<&DepthZ>, Option<&Scale2D>, Option<&Pivot2D>, Option<&RotationZ>, Option<&Rotation>, Option<&Flip>, Option<&Color>, Option<&Arc<Texture>>
        )>();
        for (_, (
            _, priority, mesh, sprite, animation, pool, position, depth, scale, pivot, rotation_z, rotation, flip, color, texture
        )) in query.iter() {
            let buffer = match priority {
                Some(p) => {
                    let i = if let Some(i) = priority_buffer_value.iter().position(|&x| x == p.0) {
                        i
                    } else {
                        let i = priority_buffer_value.len();
                        priority_buffer_value.push(p.0);
                        priority_buffer.push(PriorityBuffer::new());
                        i
                    };
                    &mut priority_buffer[i]
                }
                None => &mut no_priority,
            };
            if let Some(p) = pool {
                for (_, a) in p.animations.iter() {
                    let animation = Some(a);
                    push_to_buffer(resource_manager, buffer, sprite, animation, texture, mesh, position, depth, scale, pivot, rotation_z, rotation, flip, color);
                }
            }
            push_to_buffer(resource_manager, buffer, sprite, animation, texture, mesh, position, depth, scale, pivot, rotation_z, rotation, flip, color);
        }

        let mut indices: Vec<usize> = (0..priority_buffer_value.len()).collect();
        indices.sort_by_key(|&i| priority_buffer_value[i]);

        let priority_buffer_sorted: Vec<PriorityBuffer> = indices.iter().map(|&i| priority_buffer[i].clone()).collect();
        let mut to_render = vec![no_priority];
        to_render.extend(priority_buffer_sorted);

        let mut first_pass_color = true;
        let mut first_pass_depth = true;
        for buffer in to_render {
            let color_attachment = match first_pass_color {
                true => base_color_attachment.clone(),
                false => default_color_attachment.clone(),
            };
            let depth_ops = match first_pass_depth {
                true => base_depth_ops,
                false => default_depth_ops,
            };
            if !buffer.meshes.is_empty() {
                self.render_models(resource_manager, &mut encoder, buffer.meshes, color_attachment.clone(), depth_ops.clone());
                first_pass_color = false;
                first_pass_depth = false;
            }
            if !buffer.sprites.is_empty() {
                self.render_sprites(resource_manager, &mut encoder, buffer.sprites, color_attachment, depth_ops);
                first_pass_color = false;
                first_pass_depth = false;
            }
        }

        self.render_ui(resource_manager, &mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.text_state.atlas.trim();

        self.reset_render_state();
        
        Ok(())
    }

    fn render_ui(&mut self, resource_manager: &ResourceManager, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.text_state.viewport.update(&self.queue, glyphon::Resolution {
            width: self.config.width,
            height: self.config.height,
        });

        let mut text_areas: Vec<glyphon::TextArea> = Vec::new();
        for text_info in self.text_state.to_draw.drain(..) {
            let buffer = self.text_state.buffer_pool.using.get(text_info.buffer_index).unwrap();
            let mut offset = f32::MAX;
            for run in buffer.layout_runs() {
                for glyph in run.glyphs.iter() {
                    offset = offset.min(run.line_y - glyph.font_size);
                }
            }
            let text_area = glyphon::TextArea {
                buffer,
                left: text_info.x,
                top: text_info.y - offset,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: text_info.color,
                custom_glyphs: &[]
            };
            text_areas.push(text_area);
        }
        self.text_state.renderer.prepare(
            &self.device, 
            &self.queue, 
            &mut self.text_state.font_system, 
            &mut self.text_state.atlas, 
            &self.text_state.viewport, 
            text_areas, 
            &mut self.text_state.swash_cache,
        ).unwrap();
        self.text_state.buffer_pool.reset();

        let mut buffer_segments: [BufferSegment; 2] = [
            BufferSegment::new(ARRAY_256X256_ID),
            BufferSegment::new(ARRAY_4096X4096_ID)
        ];

        let vertex_stride = std::mem::size_of::<UIVertex>() as u32;
        let index_stride = std::mem::size_of::<u16>() as u32;
        let mut vertex_offset = 0;
        let mut index_offset = 0;
        for buffer_segment in &mut buffer_segments {
            buffer_segment.vertices_offset = vertex_offset * vertex_stride;
            buffer_segment.indices_offset = index_offset * index_stride;
            for draw_call in &self.draw_pass.draw_calls {
                let DrawCall::Mesh(mesh) = draw_call;
                if mesh.material.array_id == buffer_segment.id {
                    buffer_segment.vertices.extend(&mesh.vertices);
                    buffer_segment.indices.extend(mesh.indices.iter().map(|i| i + vertex_offset as u16));
                    vertex_offset += mesh.vertices.len() as u32;
                    index_offset += mesh.indices.len() as u32;
                }
            }
            buffer_segment.vertices_length = vertex_offset * vertex_stride - buffer_segment.vertices_offset;
            buffer_segment.indices_length = index_offset * index_stride - buffer_segment.indices_offset;
        }

        let ui_vertex_buffer = &self.ui_vertex_buffers[self.ui_current_frame];
        let ui_index_buffer = &self.ui_index_buffers[self.ui_current_frame];

        self.ui_current_frame ^= 1;

        for buffer_segment in &buffer_segments {
            self.queue.write_buffer(ui_vertex_buffer, buffer_segment.vertices_offset as u64, bytemuck::cast_slice(&buffer_segment.vertices));
            self.queue.write_buffer(ui_index_buffer, buffer_segment.indices_offset as u64, bytemuck::cast_slice(&buffer_segment.indices));
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.ui_pipeline);

            for buffer_segment in &buffer_segments {
                if buffer_segment.vertices_length != 0 && buffer_segment.indices_length != 0 {
                    let (_, bind_group) = resource_manager.texture_arrays.get(&buffer_segment.id).unwrap();
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.set_vertex_buffer(
                        0, 
                        ui_vertex_buffer.slice((buffer_segment.vertices_offset as u64)..((buffer_segment.vertices_offset + buffer_segment.vertices_length) as u64))
                    );
                    render_pass.set_index_buffer(
                        ui_index_buffer.slice((buffer_segment.indices_offset as u64)..((buffer_segment.indices_offset + buffer_segment.indices_length) as u64)), 
                        wgpu::IndexFormat::Uint16
                    );
                    render_pass.draw_indexed(0..(buffer_segment.indices_length / index_stride), 0, 0..1);
                }
            }

            self.text_state.renderer.render(&self.text_state.atlas, &self.text_state.viewport, &mut render_pass).unwrap();
        }
    }

    fn render_sprites(
        &mut self, 
        resource_manager: &ResourceManager, 
        encoder: &mut wgpu::CommandEncoder, 
        to_render: Vec<SpriteRender>,
        color_attachment: Option<wgpu::RenderPassColorAttachment<'_>>,
        depth_ops: Option<wgpu::Operations<f32>>
    ) {
        let mut buffer_segments: [BufferSegmentSpriteInstance; 2] = [
            BufferSegmentSpriteInstance::new(ARRAY_256X256_ID),
            BufferSegmentSpriteInstance::new(ARRAY_4096X4096_ID)
        ];
        let stride = std::mem::size_of::<SpriteInstance>() as u32;
        let mut offset = 0;

        for buffer_segment in &mut buffer_segments {
            buffer_segment.offset = offset * stride;

            for sprite_render in &to_render {
                if sprite_render.array_id == buffer_segment.id {
                    buffer_segment.instances.push(sprite_render.instance);
                    offset += 1;
                }
            }
            buffer_segment.length = offset * stride - buffer_segment.offset;
        }

        for buffer_segment in &buffer_segments {
            self.queue.write_buffer(&self.sprite_instance_buffer, buffer_segment.offset as u64, bytemuck::cast_slice(&buffer_segment.instances));
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[
                    color_attachment
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops,
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.sprite_pipeline);

            for buffer_segment in &buffer_segments {
                if buffer_segment.length != 0 {
                    let (_, bind_group) = resource_manager.texture_arrays.get(&buffer_segment.id).unwrap();
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.sprite_quad_vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(
                        1, 
                        self.sprite_instance_buffer.slice((buffer_segment.offset as u64)..((buffer_segment.offset + buffer_segment.length) as u64))
                    );
                    render_pass.set_index_buffer(self.sprite_quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..(SPRITE_QUAD_INDICES.len() as u32), 0, 0..(buffer_segment.length / stride));
                }
            }
        }
    }

    fn render_models(
        &mut self, 
        resource_manager: &ResourceManager, 
        encoder: &mut wgpu::CommandEncoder, 
        to_render: Vec<&ModelMesh>,
        color_attachment: Option<wgpu::RenderPassColorAttachment<'_>>,
        depth_ops: Option<wgpu::Operations<f32>>
    ) {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[
                    color_attachment
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops,
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.model_pipeline);

            let (_, bind_group) = resource_manager.texture_arrays.get(&ARRAY_256X256_ID).unwrap();
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            for mesh in to_render {
                if mesh.vertices.size() != 0 {
                    render_pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    render_pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
    
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                }
            }
        }
    }

    fn reset_render_state(&mut self) {
        self.draw_pass = DrawPass::new(None);
    }

    pub fn update_render_state(&mut self) {
        if self.camera.updated {
            self.camera_uniform.update_matrix(&self.camera);
            self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
            self.camera.updated = false;
        }
    }
}

fn push_to_buffer<'a>(
    resource_manager: &ResourceManager, buffer: &mut PriorityBuffer<'a>, sprite: Option<&Sprite>, animation: Option<&Animation>, texture: Option<&Arc<Texture>>, 
    mesh: Option<&'a ModelMesh>, position: Option<&Position2D>, depth: Option<&DepthZ>, scale: Option<&Scale2D>, pivot: Option<&Pivot2D>, rotation_z: Option<&RotationZ>,
    rotation: Option<&Rotation>, flip: Option<&Flip>, color: Option<&Color>
) {
    let fetched_texture;
    let texture: Option<&Arc<Texture>> = match animation {
        Some(a) => {
            fetched_texture = resource_manager.fetch_texture(&a.name);
            fetched_texture.as_ref()
        }
        None => texture,
    };
    if let Some(m) = mesh {
        buffer.meshes.push(m);
    }
    if sprite.is_some() {
        let sprite_render = SpriteRender::new(position, depth, scale, pivot, rotation_z, rotation, flip, color, texture, animation);
        buffer.sprites.push(sprite_render);
    }
}