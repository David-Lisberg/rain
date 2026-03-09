use crate::game::player::item::ItemType;

pub struct Inventory {
    pub slots: Vec<InventorySlot>,
}

impl Inventory {
    pub fn new(num_slots: usize) -> Self {
        Self {
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