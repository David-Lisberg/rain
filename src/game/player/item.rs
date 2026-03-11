use std::sync::Arc;

use rain::engine::{resource::ResourceManager, texture::Texture};

#[derive(Clone)]
pub enum ItemType {
    Twig,
}

impl ItemType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            ItemType::Twig => resource_manager.fetch_texture("object_twig").unwrap(),
        }
    }
}