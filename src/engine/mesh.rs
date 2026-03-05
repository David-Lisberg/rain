use std::sync::Arc;

use crate::engine::{texture::Texture, vertex::{UIVertex}};

#[derive(Clone, Debug)]
pub struct UIMesh {
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u16>,
    pub material: Arc<Texture>,
}

#[derive(Debug)]
pub struct ModelMesh {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub num_indices: u32,
    pub array_id: u32,
}