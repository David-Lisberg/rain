use std::{collections::HashMap, sync::Arc};


use anyhow::Error;
use image::GenericImageView;

use crate::engine::{animation::AnimationData, core::RainHandle, texture::{Texture, TextureArray}};

pub const ARRAY_512X512_ID: u32 = 0;
pub const ARRAY_4096X4096_ID: u32 = 1;

pub struct ResourceManager {
    pub current_id: u32,
    pub texture_arrays: HashMap<u32, (TextureArray, wgpu::BindGroup)>,
    pub textures: HashMap<String, Arc<Texture>>,
    pub animations: HashMap<String, Arc<AnimationData>>,
}

impl ResourceManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut texture_arrays: HashMap<u32, (TextureArray, wgpu::BindGroup)> = HashMap::new();

        let texture_bind_group_layout = Self::texture_bind_group_layout(device);

        let array_512x512 = TextureArray::new(device, 512, 512, 256, ARRAY_512X512_ID);
        let bind_group_512x512 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_512x512.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&array_512x512.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let array_4096x4096 = TextureArray::new(device, 4096, 4096, 8, ARRAY_4096X4096_ID);
        let bind_group_4096x4096 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_4096x4096.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&array_4096x4096.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        texture_arrays.insert(ARRAY_512X512_ID, (array_512x512, bind_group_512x512));
        texture_arrays.insert(ARRAY_4096X4096_ID, (array_4096x4096, bind_group_4096x4096));

        Self {
            current_id: 0,
            texture_arrays,
            textures: HashMap::new(),
            animations: HashMap::new(),
        }
    }

    pub fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    pub fn load_texture(&mut self, queue: &wgpu::Queue, name: String, image: &image::DynamicImage) -> Result<Arc<Texture>, Error> {
        let dimensions = image.dimensions();

        if dimensions.0 > 4096 || dimensions.1 > 4096 {
            return Err(Error::msg("Image size too large. Must be below 4096x4096."));
        }

        let (array, _) = if dimensions.0 > 512 || dimensions.1 > 512 {
            self.texture_arrays.get_mut(&ARRAY_4096X4096_ID).unwrap()
        } else {
            self.texture_arrays.get_mut(&ARRAY_512X512_ID).unwrap()
        };

        let texture = Texture::from_image(queue, array, image);
        self.textures.insert(name, Arc::clone(&texture));

        Ok(texture)
    }

    pub fn fetch_texture(&self, name: &str) -> Option<Arc<Texture>> {
        self.textures.get(name).cloned()
    }

    pub fn fetch_animation(&self, name: &str) -> Option<Arc<AnimationData>> {
        self.animations.get(name).cloned()
    }
}

impl RainHandle {
    pub fn load_texture(&mut self, name: &str, path: &str) -> Result<Arc<Texture>, Error> {
        if let Some(texture) = self.fetch_texture(name) {
            return Ok(texture);
        }

        let image = image::open(path)?;
        self.resource_manager.load_texture(&self.renderer.queue, name.to_string(), &image)
    }

    pub fn fetch_texture(&self, name: &str) -> Option<Arc<Texture>> {
        self.resource_manager.textures.get(name).cloned()
    }

    pub fn load_animation(&mut self, name: &str, path: &str) -> Result<Arc<AnimationData>, Error> {
        let json = std::fs::read_to_string(path)?;
        let animation_data: AnimationData = serde_json::from_str(&json)?;
        let animation_data = Arc::new(animation_data);
        self.load_texture(name, &animation_data.source)?;

        self.resource_manager.animations.insert(name.to_string(), Arc::clone(&animation_data));
        Ok(animation_data)
    }

    pub fn fetch_animation(&self, name: &str) -> Option<Arc<AnimationData>> {
        self.resource_manager.animations.get(name).cloned()
    }
}