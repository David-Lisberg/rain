use std::ops::Range;

use crate::game::player::item::ItemType;

pub const INVENTORY_SLOTS_PLAYER: usize = 36;
pub const INVENTORY_SLOTS_WIDTH: usize = 9;
pub const INVENTORY_SLOTS_HOTBAR: Range<usize> = 0..9;
pub const INVENTORY_SLOTS_INVENTORY: Range<usize> = 9..36;

pub struct Inventory {
    pub open: bool,
    pub slots: Vec<InventorySlot>,
}

impl Inventory {
    pub fn new(num_slots: usize) -> Self {
        Self {
            open: false,
            slots: vec![InventorySlot::new(); num_slots],
        }
    }
}

#[derive(Clone)]
pub struct InventorySlot {
    pub item_type: Option<ItemType>,
    pub quantity: u32,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self {
            item_type: None,
            quantity: 0,
        }
    }
}