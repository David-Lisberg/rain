use std::ops::Range;

use crate::game::player::item::Item;

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

    pub fn add_item(&mut self, item: Item, quantity: u32) {
        let mut item_found = false;
        for slot in self.slots.iter_mut() {
            if let Some(current_item) = &slot.item {
                if item == *current_item {
                    slot.quantity += quantity;
                    item_found = true;
                    break;
                }
            }
        }
        if !item_found {
            for slot in self.slots.iter_mut() {
                if slot.item.is_none() {
                    slot.item = Some(item);
                    slot.quantity += quantity;
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct InventorySlot {
    pub item: Option<Item>,
    pub quantity: u32,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self {
            item: None,
            quantity: 0,
        }
    }
}