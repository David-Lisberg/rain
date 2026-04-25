use std::sync::Arc;

use rain::engine::{resource::ResourceManager, texture::Texture};

#[derive(Clone, PartialEq)]
pub struct Item {
    pub _type: ItemType,
    pub category: ItemCategory,
}

impl Item {
    pub fn new(item_type: ItemType) -> Self {
        let category = match item_type {
            ItemType::FlintHatchet => ItemCategory::Tool(1, 1, 5.0),
            _ => ItemCategory::Other,
        };

        Self { _type: item_type, category }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ItemType {
    Twig,
    Grass,
    Stone,
    Twine,
    Sling,
    Flint,
    FlintHatchet,
    Wood,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemCategory {
    Tool(i32, i32, f32),
    Other,
}

impl ItemType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            ItemType::Twig => resource_manager.fetch_texture("object_twig").unwrap(),
            ItemType::Grass => resource_manager.fetch_texture("object_grass").unwrap(),
            ItemType::Stone => resource_manager.fetch_texture("object_stone").unwrap(),
            ItemType::Flint => resource_manager.fetch_texture("object_flint").unwrap(),
            ItemType::FlintHatchet => resource_manager.fetch_texture("flint_hatchet").unwrap(),
            ItemType::Wood => resource_manager.fetch_texture("item_wood").unwrap(),
            ItemType::Twine => resource_manager.fetch_texture("item_twine").unwrap(),
            ItemType::Sling => resource_manager.fetch_texture("item_sling").unwrap(),
        }
    }

    pub fn stack_size_max(&self) -> u32 {
        match self {
            ItemType::Sling => 1,
            ItemType::FlintHatchet => 1,
            _ => 100,
        }
    }
}