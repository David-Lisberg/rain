use std::{collections::HashMap, ops::Range};

use hecs::Entity;
use lgui::element::{EBuilder, Scale};
use rain::engine::core::RainHandle;
use rain::engine::input::{KeyboardKey, MouseButton};

use crate::game::player::movement::Player;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State};
use crate::game::core::ui::{INVENTORY_SLOT_GAP, INVENTORY_SLOT_HEIGHT, INVENTORY_SLOT_SIZE};
use crate::game::player::crafting::{Recipe, check_available_recipes, craft_item};
use crate::game::player::item::{Item, ItemType};


pub const INVENTORY_SLOTS_PLAYER: usize = 36;
pub const INVENTORY_SLOTS_WIDTH: usize = 9;
pub const INVENTORY_SLOTS_HOTBAR: Range<usize> = 0..9;
pub const INVENTORY_SLOTS_INVENTORY: Range<usize> = 9..36;

pub const UI_INVENTORY_MAIN: InventoryUI = InventoryUI {
    rows: 3,
    columns: 9,
    row_gap: INVENTORY_SLOT_GAP,
    column_gap: INVENTORY_SLOT_GAP,
    slot_size: INVENTORY_SLOT_SIZE,
    slot_texture: "inventory_slot",
    background_texture: None,
    background_gap: None,
};
pub const UI_INVENTORY_HOTBAR: InventoryUI = InventoryUI {
    rows: 1,
    columns: 9,
    row_gap: INVENTORY_SLOT_GAP,
    column_gap: INVENTORY_SLOT_GAP,
    slot_size: INVENTORY_SLOT_SIZE,
    slot_texture: "inventory_slot",
    background_texture: None,
    background_gap: None,
};

pub struct Inventory {
    pub slots: Vec<InventorySlot>,
}

pub struct InventoryScreen {
    pub panels: Vec<InventoryPanel>,
    pub selection: Vec<InventorySelection>,
}

impl InventoryScreen {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            selection: Vec::new(),
        }
    }
}

pub struct InventoryPanel {
    pub inventory: Entity,
    pub slots: Option<Range<usize>>,
    pub gap: f32,
    pub ui: InventoryUI,
}

pub struct InventoryUI {
    pub rows: i32,
    pub columns: i32,
    pub row_gap: f32,
    pub column_gap: f32,
    pub slot_size: f32,
    pub slot_texture: &'static str,
    pub background_texture: Option<&'static str>,
    pub background_gap: Option<f32>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct InventorySelection {
    pub inventory: Entity,
    pub slot: usize,
}

impl InventorySelection {
    pub fn new(inventory: Entity, slot: usize) -> Self {
        Self { inventory, slot }
    }
}

#[derive(Clone)]
pub struct PlayerInventory {
    pub open: bool,
    pub just_opened: bool,
    pub display_recipes: bool,
    pub available_recipes: Vec<Recipe>,
    pub selected_hotbar: usize,
}

#[derive(Clone)]
pub struct InventoryHover(pub InventorySelection);
#[derive(Clone)]
pub struct CraftHover(pub usize);

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            open: false,
            just_opened: false,
            display_recipes: false,
            available_recipes: Vec::new(),
            selected_hotbar: 0,
        }
    }
}

impl Inventory {
    pub fn new(num_slots: usize) -> Self {
        Self {
            slots: vec![InventorySlot::new(); num_slots]
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

    pub fn collect_items(&self, selected: Vec<usize>) -> Vec<(ItemType, i32)> {
        let mut input_map: HashMap<ItemType, i32> = HashMap::new();
        if !selected.is_empty() {
            for select in selected.iter() {
                let slot = self.slots.get(*select).unwrap();
                if let Some(item) = &slot.item {
                    *input_map.entry(item._type.clone()).or_insert(0) += slot.quantity;
                }
            }
        } else {
            for slot in self.slots.iter() {
                if let Some(item) = &slot.item {
                    *input_map.entry(item._type.clone()).or_insert(0) += slot.quantity;
                }
            }
        }
        input_map.into_iter().collect()
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
    let point = handle.mouse_position();
    let mut y_pointer = INVENTORY_SLOT_HEIGHT;
    let mut inventory_select: Option<InventorySelection> = None;

    let mut player_info: Option<(PlayerInventory, Entity)> = None;
    for (e, (_, p)) in handle.world.query::<(&Player, &PlayerInventory)>().iter() {
        player_info = Some((p.clone(), e));
    }
    let (mut player_inventory, player_entity) = player_info.unwrap();

    let right_button_released = handle.is_button_released(MouseButton::Right);
    let left_button_released = handle.is_button_released(MouseButton::Left);
    let shift_pressed = handle.is_key_pressed(KeyboardKey::ShiftLeft);

    let mut to_add_inventory_hover: Vec<(Entity, InventorySelection)> = Vec::new();
    let mut to_remove_inventory_hover: Vec<Entity> = Vec::new();
    let mut to_add_craft_hover: Vec<(Entity, usize)> = Vec::new();
    let mut to_remove_craft_hover: Vec<Entity> = Vec::new();

    let mut updated = false;

    if player_inventory.open {
        if player_inventory.just_opened {
            if let Some(inventory) = handle.world.query_one::<&Inventory>(player_entity).unwrap().get() {
                player_inventory.just_opened = false;

                let selected: Vec<usize> = state.inventory_screen.selection.iter().map(|x| x.slot).collect();
                let inputs = inventory.collect_items(selected);
                player_inventory.available_recipes = check_available_recipes(state, &inputs)
            }
        }

        for panel in state.inventory_screen.panels.iter() {
            y_pointer -= panel.gap;
            let slots_width = panel.ui.columns as f32 * panel.ui.slot_size + (panel.ui.columns - 1) as f32 * panel.ui.column_gap;
            let x_pointer = (SCREEN_WIDTH - slots_width) / 2.0;
    
            let range = panel.slots.clone().unwrap();
            let slots = 0..(range.end - range.start);
            for i in slots {
                if state.gui.button(point, EBuilder::new(0.0, 0.0)
                    .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(
                        x_pointer + (i as i32 % panel.ui.columns) as f32 * (panel.ui.slot_size + panel.ui.column_gap), 
                        y_pointer - (i as i32 / panel.ui.columns) as f32 * (panel.ui.slot_size + panel.ui.row_gap)
                    ).rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE)
                        .build())
                    .build()) {
                    inventory_select = Some(InventorySelection::new(panel.inventory, i + range.start));
                }
            }
            y_pointer -= panel.ui.rows as f32 * (panel.ui.slot_size + panel.ui.row_gap);
        }
    
        if let Some(selection) = inventory_select {
            if left_button_released {
                if let Some(selected) = state.inventory_screen.selection.iter().position(|&x| x == selection) {
                    state.inventory_screen.selection.remove(selected);
                } else {
                    if !shift_pressed {
                        state.inventory_screen.selection.clear();
                    }
                    state.inventory_screen.selection.push(selection);
                }
                updated = true;
            } else if right_button_released {
                if state.inventory_screen.selection.len() == 1 {
                    let selected = state.inventory_screen.selection[0];
                    state.inventory_screen.selection[0] = selection;
                    if let Ok(inventory) = handle.world.query_one_mut::<&mut Inventory>(selection.inventory) {
                        inventory.slots.swap(selected.slot, selection.slot);
                    }
                }
            }
        } else if !updated && (right_button_released || left_button_released) {
            state.inventory_screen.selection.clear();
        }

        let mut to_craft: Option<usize> = None;
        if player_inventory.display_recipes {
            let panel = state.inventory_screen.panels.first().unwrap();
            let num_recipes = player_inventory.available_recipes.len() as f32;
            let recipes_width = num_recipes * panel.ui.slot_size + (num_recipes - 1.0) * panel.ui.column_gap;
            let x_pointer = (SCREEN_WIDTH - recipes_width) / 2.0;
            y_pointer -= panel.ui.row_gap; 
    
            for i in 0..player_inventory.available_recipes.len() {
                if state.gui.button(point, EBuilder::new(0.0, 0.0)
                    .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .scale(Scale::NormalShift)
                    .sub_element(|| EBuilder::new(x_pointer + i as f32 * (panel.ui.slot_size + panel.ui.column_gap), y_pointer)
                        .rect(panel.ui.slot_size, panel.ui.slot_size)
                        .build())
                    .build()) {
                    to_craft = Some(i);
                }
            }
        }

        if let Ok((inventory, player_inventory_update)) = handle.world.query_one_mut::<(
            &mut Inventory, &mut PlayerInventory
        )>(player_entity) {
            if left_button_released {
                if let Some(index) = to_craft {
                    craft_item(inventory, &player_inventory.available_recipes[index]);
                    updated = true;
                }
            }
            if updated {
                let selected: Vec<usize> = state.inventory_screen.selection.iter().map(|x| x.slot).collect();
                let inputs = inventory.collect_items(selected);
                player_inventory.available_recipes = check_available_recipes(state, &inputs);
            }
            *player_inventory_update = player_inventory;
        }

        match (to_craft, updated) {
            (Some(i), false) => to_add_craft_hover.push((player_entity, i)),
            _ => to_remove_craft_hover.push(player_entity),
        }
        match inventory_select {
            Some(selection) => to_add_inventory_hover.push((player_entity, selection)),
            None => to_remove_inventory_hover.push(player_entity),
        }
    }

    for (e, i) in to_add_inventory_hover {
        handle.world.insert_one(e, InventoryHover(i)).unwrap();
    }
    for e in to_remove_inventory_hover {
        let _ = handle.world.remove_one::<InventoryHover>(e).is_ok();
    }
    for (e, i) in to_add_craft_hover {
        handle.world.insert_one(e, CraftHover(i)).unwrap();
    }
    for e in to_remove_craft_hover {
        let _ = handle.world.remove_one::<CraftHover>(e).is_ok();
    }
}

pub fn setup_inventory_ui(state: &mut State, player_entity: Entity) {
    state.inventory_screen.panels.push(InventoryPanel {
        inventory: player_entity,
        slots: Some(0..9),
        gap: 0.0,
        ui: UI_INVENTORY_HOTBAR,
    });
} 