use std::collections::HashMap;

use glam::Vec2;
use hecs::Entity;
use rain::engine::component::*;
use rain::engine::core::RainHandle;
use serde::Deserialize;

use crate::{DEPTH_PLAYER, State};
use crate::game::player::inventory::Inventory;
use crate::game::player::movement::Player;
use crate::game::utility::timer::Timer;


const ITEM_PICKUP_RANGE: f32 = 1.0;

pub type ItemRegistry = HashMap<ItemType, ItemData>;
pub struct TimerPickup(pub Timer);

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub _type: ItemType,
    pub category: ItemCategory,
}

#[derive(Deserialize)]
pub struct ItemData {
    pub name: String,
    pub texture: String,
}

impl Item {
    pub fn new(item_type: ItemType) -> Self {
        let category = match item_type {
            ItemType::FlintHatchet => ItemCategory::Tool(ToolType::Axe, 1, 1, 4.0),
            ItemType::BoneHatchet => ItemCategory::Tool(ToolType::Axe, 1, 1, 3.0),
            ItemType::StonePickaxe => ItemCategory::Tool(ToolType::Pickaxe, 1, 1, 5.0),
            ItemType::BoneShovel => ItemCategory::Tool(ToolType::Shovel, 1, 1, 3.0),
            ItemType::WoodShovel => ItemCategory::Tool(ToolType::Shovel, 2, 1, 2.0),
            _ => ItemCategory::Other,
        };

        Self { _type: item_type, category }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Twig,
    Grass,
    Stone,
    Twine,
    Sling,
    Flint,
    FlintHatchet,
    BoneHatchet,
    StonePickaxe,
    Wood,
    WoodPlanks,
    CoatiPelt,
    SquirrelPelt,
    SmallBone,
    BonePlate,
    BoneShovel,
    WoodShovel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolType {
    Axe,
    Pickaxe,
    Shovel,
    None,
}

impl ToolType {
    pub fn can_break(&self, other: ToolType) -> bool {
        if other == ToolType::None {
            true
        } else {
            *self == other
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemCategory {
    Tool(ToolType, i32, i32, f32), /* type, break level, hit ticks, damage */
    Other,
}

impl ItemType {
    pub fn stack_size_max(&self) -> i32 {
        match self {
            ItemType::Sling => 1,
            ItemType::FlintHatchet => 1,
            ItemType::BoneHatchet => 1,
            ItemType::StonePickaxe => 1,
            ItemType::BoneShovel => 1,
            ItemType::WoodShovel => 1,
            _ => 100,
        }
    }
}

#[derive(Clone)]
pub struct ItemDrop {
    item: Item,
    quantity: i32,
}

pub fn spawn_item_drop(handle: &mut RainHandle, state: &mut State, position: Position2D, item: Item, quantity: i32) {
    let item_data = state.item_registry.get(&item._type).unwrap();
    let texture = handle.fetch_texture(&item_data.texture).unwrap();
    let item_drop = ItemDrop { item, quantity };
    handle.world.spawn((Sprite, Visible, texture, item_drop, position, Scale2D(Vec2::new(0.3, 0.3)), DepthZ(DEPTH_PLAYER), Priority(1)));
}

pub fn spawn_item_drop_with_timer(handle: &mut RainHandle, state: &mut State, position: Position2D, item: Item, quantity: i32, time: f32) {
    let item_data = state.item_registry.get(&item._type).unwrap();
    let texture = handle.fetch_texture(&item_data.texture).unwrap();
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

pub fn drop_current_item(handle: &mut RainHandle, state: &mut State, drop_all: bool) {
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
        spawn_item_drop_with_timer(handle, state, position, item, quantity, 2.0);
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