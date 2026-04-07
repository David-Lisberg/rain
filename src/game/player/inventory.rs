use std::ops::Range;

use lgui::element::{EBuilder, Element, Scale, Shape};
use rain::engine::core::RainHandle;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State, game::{core::ui::*, player::{item::Item, movement::Player}}};

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
    pub selected: Vec<usize>,
    pub quantity: u32,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self {
            item: None,
            selected: Vec::new(),
            quantity: 0,
        }
    }
}

pub fn system_inventory_select(handle: &mut RainHandle, state: &mut State) {
    let num_slots = INVENTORY_SLOTS_HOTBAR.len() as f32;
    let slots_width = num_slots * INVENTORY_SLOT_SIZE + (num_slots - 1.0) * INVENTORY_SLOT_GAP;
    let start = (SCREEN_WIDTH - slots_width) / 2.0; 
    let point = handle.mouse_position();

    for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
        if inventory.open {
            for i in INVENTORY_SLOTS_HOTBAR {
                let x = start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                state.gui.button(EBuilder::new(0.0, 0.0)
                    .shape(Shape::Rect(SCREEN_WIDTH, SCREEN_HEIGHT))
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(x, INVENTORY_SLOT_HEIGHT)
                        .shape(Shape::Rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
                        .build())
                    .build(), point, handle);
            }
        }
    }
}