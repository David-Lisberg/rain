use std::sync::Arc;

use rain::engine::{component::{Direction, Position2D, Scale2D}, core::RainHandle, resource::ResourceManager, texture::Texture};

use crate::{State, game::player::{inventory::Inventory, movement::Player}};

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

// pub fn item_pickup(handle: &mut RainHandle, state: &mut State) {
//     let mut query = handle.world.query_mut::<(&Player, &Position2D, &Scale2D, &Direction, &mut Inventory)>();
//     for (_, (_, position, size, direction, inventory)) in query {
//         let collider_position = match direction {
//             Di
//         };
//     }
// }