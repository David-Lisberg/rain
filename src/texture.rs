use std::sync::Arc;
use image::{ImageBuffer, Rgba, GenericImageView};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn white_pixel(device: &wgpu::Device, queue: &wgpu::Queue) -> Arc<Texture> {
        let raw: Vec<u8> = vec![255, 255, 255, 255];
        let image: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(1, 1, raw).unwrap();
        let image = image::DynamicImage::ImageRgba8(image);
        Self::from_image(device, queue, &image)
    }

    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> Arc<Texture> {
        let image = image::load_from_memory(bytes).unwrap();
        Self::from_image(device, queue, &image)
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: &image::DynamicImage) -> Arc<Texture> {
        let image = image.to_rgba8();
        let dimensions = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Arc::new(Texture {
            texture,
            view,
            sampler,
        })
    }
}