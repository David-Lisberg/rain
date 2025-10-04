use std::sync::Arc;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

use crate::{core::RainHandle, utility::image::resize_and_pad};

#[derive(Debug)]
pub struct Texture {
    pub array_id: u32,
    pub index: u32,
    pub uv: [f32; 2],
}

impl Texture {
    pub fn white_pixel() -> DynamicImage {
        let raw: Vec<u8> = vec![255, 255, 255, 255];
        let image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(1, 1, raw).unwrap();
        image::DynamicImage::ImageRgba8(image)
    }

    pub fn from_bytes(queue: &wgpu::Queue, array: &mut TextureArray, bytes: &[u8]) -> Arc<Texture> {
        let image = image::load_from_memory(bytes).unwrap();
        Self::from_image(queue, array, &image)
    }

    pub fn from_image(queue: &wgpu::Queue, array: &mut TextureArray, image: &image::DynamicImage) -> Arc<Texture> {
        let image = image.to_rgba8();
        let dimensions = image.dimensions();

        let image = resize_and_pad(image, array.width, array.height);

        let texture_size = wgpu::Extent3d {
            width: array.width,
            height: array.height,
            depth_or_array_layers: 1,
        };
        // let texture = device.create_texture(&wgpu::TextureDescriptor {
        //     size: texture_size,
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
        //     usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //     label: Some("diffuse_texture"),
        //     view_formats: &[],
        // });

        if array.current >= array.layers {
            panic!("Error: Attempting to write more textures than array can hold.")
        }

        let index = array.current;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &array.array,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: index,
                },
                aspect: wgpu::TextureAspect::All,
            },
            image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * array.width),
                rows_per_image: Some(array.height),
            },
            texture_size,
        );

        array.current += 1;

        Arc::new(Texture {
            // texture,
            index,
            array_id: array.id,
            uv: [(dimensions.0 as f32 / array.width as f32), (dimensions.1 as f32 / array.height as f32)]
        })
    }
}

pub struct TextureArray {
    pub array: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub current: u32,
    pub id: u32,
}

impl TextureArray {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, layers: u32, id: u32) -> TextureArray {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers,
        };

        let array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("texture_{}x{}_array", width, height)),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = array.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("texture_{}x{}_array_view", width, height)),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        TextureArray {
            array,
            view,
            sampler,
            width,
            height,
            layers,
            current: 0,
            id,
        }
    }
}