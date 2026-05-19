use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::*, core::RainHandle, resource::ResourceManager, texture::Texture};

use crate::{DEPTH_PLAYER, game::{player::{inventory::Inventory, movement::Player}, utility::timer::Timer}};

const ITEM_PICKUP_RANGE: f32 = 1.0;

pub struct TimerPickup(pub Timer);

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
    CoatiPelt,
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
            ItemType::CoatiPelt => resource_manager.fetch_texture("item_coati_pelt").unwrap(),
        }
    }

    pub fn stack_size_max(&self) -> i32 {
        match self {
            ItemType::Sling => 1,
            ItemType::FlintHatchet => 1,
            _ => 100,
        }
    }
}

#[derive(Clone)]
pub struct ItemDrop {
    item: Item,
    quantity: i32,
}

pub fn spawn_item_drop(handle: &mut RainHandle, position: Position2D, item: Item, quantity: i32) {
    let texture = item._type.fetch_texture(&handle.resource_manager);
    let item_drop = ItemDrop { item, quantity };
    handle.world.spawn((Sprite, Visible, texture, item_drop, position, Scale2D(Vec2::new(0.3, 0.3)), DepthZ(DEPTH_PLAYER), Priority(1)));
}

pub fn spawn_item_drop_with_timer(handle: &mut RainHandle, position: Position2D, item: Item, quantity: i32, time: f32) {
    let texture = item._type.fetch_texture(&handle.resource_manager);
    let item_drop = ItemDrop { item, quantity };
    handle.world.spawn((
        Sprite, Visible, texture, item_drop, position, Scale2D(Vec2::new(0.3, 0.3)), DepthZ(DEPTH_PLAYER), Priority(1), TimerPickup(Timer::new(time))),
    );
}

pub fn system_item_drop_pickup(handle: &mut RainHandle) {
    let mut item_drops: Vec<(Entity, ItemDrop, Position2D)> = Vec::new();
    let mut to_despawn: Vec<Entity> = Vec::new();
    let mut to_update_quantity: Vec<(Entity, i32)> = Vec::new();
    for (e, (item_drop, position)) in handle.world.query::<(&ItemDrop, &Position2D)>().without::<&TimerPickup>().iter() {
        item_drops.push((e, item_drop.clone(), position.clone()));
    }
    for (_, (_, inventory, position)) in handle.world.query_mut::<(&Player, &mut Inventory, &Position2D)>() {
        for (e, item_drop, item_drop_position) in item_drops.iter() {
            let distance = (item_drop_position.0 - position.0).length();
            if distance <= ITEM_PICKUP_RANGE {
                let remaining = inventory.add_item(item_drop.item.clone(), item_drop.quantity);
                if remaining > 0 {
                    to_update_quantity.push((*e, remaining));
                } else {
                    to_despawn.push(*e)
                }
            }
        }
    }
    for (e, quantity) in to_update_quantity {
        let mut item_drop = handle.world.get::<&mut ItemDrop>(e).unwrap();
        item_drop.quantity = quantity;
    }
    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}

pub fn drop_current_item(handle: &mut RainHandle, drop_all: bool) {
    let mut to_spawn: Vec<(Position2D, Item, i32)> = Vec::new();
    for (_, (_, position, inventory)) in handle.world.query_mut::<(&Player, &Position2D, &mut Inventory)>() {
        let index = if inventory.open {
            if let Some(i) = inventory.selected.get(0) {
                *i
            } else {
                return;
            }
        } else {
            inventory.selected_hotbar
        };
        if let Some(item) = &inventory.slots[index].item {
            let item = item.clone();
            if drop_all {
                let quantity = inventory.slots[index].quantity;
                inventory.remove_item_from_slot(index, quantity);
                to_spawn.push((position.clone(), item.clone(), quantity));
            } else {
                inventory.remove_item_from_slot(index, 1);
                to_spawn.push((position.clone(), item.clone(), 1));
            }
        }
    }

    for (position, item, quantity) in to_spawn {
        spawn_item_drop_with_timer(handle, position, item, quantity, 2.0);
    }
}

pub fn system_timer_pickup(handle: &mut RainHandle) {
    let mut to_remove: Vec<Entity> = Vec::new();

    for (e, timer_pickup) in handle.world.query_mut::<&mut TimerPickup>() {
        if timer_pickup.0.step(handle.delta_time) {
            to_remove.push(e);
        }
    }

    for e in to_remove {
        handle.world.remove_one::<TimerPickup>(e).unwrap();
    }
}