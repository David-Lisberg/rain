use rain::{engine::core::RainHandle, lgui::element::{ElementBuilder, Scale, Shape}};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State, game::player::{inventory::*, movement::Player}};

const INVENTORY_SLOT_SIZE: f32 = 48.0;
const INVENTORY_SLOT_HEIGHT: f32 = SCREEN_HEIGHT - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
const INVENTORY_SLOT_GAP: f32 = 10.0;

pub fn render_ui(handle: &mut RainHandle, state: &mut State) {
    let current_width = handle.renderer.config.width as f32;
    let current_height = handle.renderer.config.height as f32;

    state.manager.begin_immediate_retain_layout(current_width, current_height);

    render_inventory(handle, state);

    state.manager.end_immediate(handle);
}

fn render_inventory(handle: &mut RainHandle, state: &mut State) {
    let num_slots = INVENTORY_SLOTS_HOTBAR.len() as f32;
    let slots_width = num_slots * INVENTORY_SLOT_SIZE + (num_slots - 1.0) * INVENTORY_SLOT_GAP;
    let start = (SCREEN_WIDTH - slots_width) / 2.0; 
    let border_width = INVENTORY_SLOT_SIZE / 8.0;


    state.manager.sub_layout_immediate("root", 0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, Scale::SquareY, 
        false, false, "inventory");

    for (_, (_, inventory)) in handle.world.query::<(&Player, &Inventory)>().iter() {

        for i in INVENTORY_SLOTS_HOTBAR {
            state.manager.element_immediate(&ElementBuilder::new(start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP), INVENTORY_SLOT_HEIGHT, 
                Shape::Rectangle(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
                .texture(handle.fetch_texture("inventory_slot").unwrap())
                .build(), "inventory");
            if let Some(item_type) = &inventory.slots[i].item_type {
                state.manager.element_immediate(&ElementBuilder::new(start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP) + border_width, 
                    INVENTORY_SLOT_HEIGHT + border_width, 
                    Shape::Rectangle(INVENTORY_SLOT_SIZE - border_width * 2.0, INVENTORY_SLOT_SIZE - border_width * 2.0))
                    .texture(item_type.fetch_texture(&mut handle.resource_manager))
                    .build(), "inventory");
            }
        }
    }
}