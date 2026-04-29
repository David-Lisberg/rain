use std::{collections::HashMap, ops::Range};

use lgui::element::{EBuilder, Scale};
use rain::engine::{core::RainHandle, input::MouseButton};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State, game::{core::ui::*, player::{crafting::{Recipe, check_available_recipes, craft_item}, item::{Item, ItemType}, movement::Player}}};

pub const INVENTORY_SLOTS_PLAYER: usize = 36;
pub const INVENTORY_SLOTS_WIDTH: usize = 9;
pub const INVENTORY_SLOTS_HOTBAR: Range<usize> = 0..9;
pub const INVENTORY_SLOTS_INVENTORY: Range<usize> = 9..36;

pub struct Inventory {
    pub open: bool,
    pub slots: Vec<InventorySlot>,
    pub selected: Vec<usize>,
    pub selected_hotbar: usize,
    pub available_recipes: Vec<Recipe>,
}

impl Inventory {
    pub fn new(num_slots: usize) -> Self {
        Self {
            open: false,
            slots: vec![InventorySlot::new(); num_slots],
            selected: Vec::new(),
            selected_hotbar: 0,
            available_recipes: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: Item, mut quantity: i32) -> i32 {
        let mut item_found = false;
        let stack_size_max = item._type.stack_size_max();
        for slot in self.slots.iter_mut() {
            if let Some(current_item) = &slot.item {
                if item == *current_item {
                    if slot.quantity + quantity > stack_size_max {
                        quantity -= stack_size_max - slot.quantity;
                        slot.quantity = stack_size_max;
                    } else {
                        slot.quantity += quantity;
                        quantity = 0;
                        item_found = true;
                        break;
                    }
                }
            }
        }
        if !item_found {
            for slot in self.slots.iter_mut() {
                if slot.item.is_none() {
                    slot.item = Some(item);
                    slot.quantity += quantity;
                    quantity = 0;
                    break;
                }
            }
        }
        quantity
    }

    pub fn remove_item(&mut self, item_type: ItemType, mut quantity: i32) -> bool {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut quantity_remaining: i32 = quantity;
        let mut success = false;
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(item) = &slot.item {
                if item._type == item_type {
                    to_remove.push(i);
                    if slot.quantity >= quantity_remaining {
                        success = true;
                        break;
                    } else {
                        quantity_remaining -= slot.quantity;
                    }
                }
            }
        }

        if !success {
            return false;
        }

        for i in to_remove {
            let slot = self.slots.get_mut(i).unwrap();
            if slot.quantity >= quantity {
                slot.quantity -= quantity;
                if slot.quantity == 0 {
                    slot.item = None;
                }
                break;
            } else {
                quantity -= slot.quantity;
                slot.quantity = 0;
                slot.item = None;
            }
        }

        true
    }

    pub fn remove_item_from_slot(&mut self, index: usize, quantity: i32) -> bool {
        let slot = self.slots.get_mut(index).unwrap();
        if slot.quantity >= quantity {
            slot.quantity -= quantity;
            if slot.quantity == 0 {
                slot.item = None;
            }
            true
        } else {
            false
        }
    }

    pub fn search_item(&self, item_type: ItemType, quantity: i32) -> Option<Vec<usize>> {
        let mut quantity_found = 0;
        let mut slots_found: Vec<usize> = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(item) = &slot.item {
                if item._type == item_type {
                    quantity_found += slot.quantity;
                    slots_found.push(i);
                }
            }
        }
        if quantity_found >= quantity {
            Some(slots_found)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct InventorySlot {
    pub item: Option<Item>,
    pub quantity: i32,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self {
            item: None,
            quantity: 0,
        }
    }
}

pub fn system_inventory_interface(handle: &mut RainHandle, state: &mut State) {
    let num_slots = INVENTORY_SLOTS_HOTBAR.len() as f32;
    let slots_width = num_slots * INVENTORY_SLOT_SIZE + (num_slots - 1.0) * INVENTORY_SLOT_GAP;
    let start = (SCREEN_WIDTH - slots_width) / 2.0; 
    let point = handle.mouse_position();
    let mut updated = false;

    if !handle.is_button_released(MouseButton::Left) {
        return;
    }

    for (_, (_, inventory)) in handle.world.query_mut::<(&Player, &mut Inventory)>() {
        if inventory.open {
            for i in INVENTORY_SLOTS_HOTBAR {
                let x = start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                if state.gui.button(point, EBuilder::new(0.0, 0.0)
                    .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(x, INVENTORY_SLOT_HEIGHT)
                        .rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE)
                        .build())
                    .build()) {
                    if let Some(index) = inventory.selected.iter().position(|&x| x == i) {
                        inventory.selected.remove(index);
                    } else {
                        inventory.selected.push(i);
                    }
                    updated = true;
                }
            }
            for i in INVENTORY_SLOTS_INVENTORY {
                let x = start + (i % INVENTORY_SLOTS_WIDTH) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                let y = INVENTORY_SLOT_HEIGHT - INVENTORY_GAP - (i / INVENTORY_SLOTS_WIDTH - 1) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                if state.gui.button(point, EBuilder::new(0.0, 0.0)
                    .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(x, y)
                        .rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE)
                        .build())
                    .build()) {
                    if let Some(index) = inventory.selected.iter().position(|&x| x == i) {
                        inventory.selected.remove(index);
                    } else {
                        inventory.selected.push(i);
                    }
                    updated = true;
                }
            }
            let num_recipes = inventory.available_recipes.len() as f32;
            let recipes_width = num_recipes * INVENTORY_SLOT_SIZE + (num_recipes - 1.0) * INVENTORY_SLOT_GAP;
            let start = (SCREEN_WIDTH - recipes_width) / 2.0;
            let y = INVENTORY_SLOT_HEIGHT - INVENTORY_GAP - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP) * 3.2;
            for (i, recipe) in inventory.available_recipes.clone().iter().enumerate() {
                let x = start + (i % INVENTORY_SLOTS_WIDTH) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                if state.gui.button(point, EBuilder::new(0.0, 0.0)
                    .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(x, y)
                        .rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE)
                        .build())
                    .build()) {
                    craft_item(inventory, recipe);
                    updated = true;
                }
            }
            if updated {
                let mut input_map: HashMap<ItemType, i32> = HashMap::new();
                for selected in inventory.selected.iter() {
                    let slot = inventory.slots.get(*selected).unwrap();
                    if let Some(item) = &slot.item {
                        *input_map.entry(item._type.clone()).or_insert(0) += slot.quantity;
                    }
                }
                let inputs: Vec<(ItemType, i32)> = input_map.into_iter().collect();
                inventory.available_recipes = check_available_recipes(&inputs);
            }
        }
    }
}