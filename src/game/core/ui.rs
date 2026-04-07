use rain::engine::core::RainHandle;
use lgui::element::*;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State, game::player::{inventory::*, movement::Player}};


pub const INVENTORY_SLOT_SIZE: f32 = 54.0;
pub const INVENTORY_SLOT_HEIGHT: f32 = SCREEN_HEIGHT - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
pub const INVENTORY_SLOT_GAP: f32 = 10.0;
pub const INVENTORY_GAP: f32 = 250.0;
pub const INVENTORY_SLOT_FONT_SIZE: u32 = 15;

pub fn render_ui(handle: &mut RainHandle, state: &mut State) {
    let current_width = handle.renderer.config.width as f32;
    let current_height = handle.renderer.config.height as f32;

    state.gui.begin(current_width, current_height);

    render_inventory(handle, state);

    state.gui.finish(handle);
}

fn render_inventory(handle: &mut RainHandle, state: &mut State) {
    let num_slots = INVENTORY_SLOTS_HOTBAR.len() as f32;
    let slots_width = num_slots * INVENTORY_SLOT_SIZE + (num_slots - 1.0) * INVENTORY_SLOT_GAP;
    let start = (SCREEN_WIDTH - slots_width) / 2.0; 
    let border_width = INVENTORY_SLOT_SIZE / 8.0;

    let mut element = EBuilder::new(0.0, 0.0)
        .shape(Shape::Rect(SCREEN_WIDTH, SCREEN_HEIGHT))
        .scale(Scale::NormalShift)
        .visible(false);

    for (_, (_, inventory)) in handle.world.query::<(&Player, &Inventory)>().iter() {

        for i in INVENTORY_SLOTS_HOTBAR {
            let x = start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
            let mut slot_element = EBuilder::new(x, INVENTORY_SLOT_HEIGHT)
                .shape(Shape::Rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
                .texture(handle.fetch_texture("inventory_slot").unwrap());
            if let Some(item) = &inventory.slots[i].item {
                slot_element.sub_element_ex(||
                    EBuilder::new(border_width, border_width)
                        .shape(Shape::Rect(INVENTORY_SLOT_SIZE - border_width * 2.0, INVENTORY_SLOT_SIZE - border_width * 2.0))
                        .texture(item._type.fetch_texture(&mut handle.resource_manager))
                        .build()
                );
                if inventory.slots[i].quantity > 1 {
                    slot_element.sub_element_ex(||
                        EBuilder::new(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE - INVENTORY_SLOT_FONT_SIZE as f32)
                            .shape(Shape::Text(format!("{}", inventory.slots[i].quantity), INVENTORY_SLOT_FONT_SIZE, Allignment::Right))
                            .build()
                    );
                }
            }
            element.sub_element_ex(|| slot_element.build());
        }

        if inventory.open {
            for i in INVENTORY_SLOTS_INVENTORY {
                let x = start + (i % INVENTORY_SLOTS_WIDTH) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                let y = INVENTORY_SLOT_HEIGHT - INVENTORY_GAP - (i / INVENTORY_SLOTS_WIDTH - 1) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                let mut slot_element = EBuilder::new(x, y)
                    .shape(Shape::Rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
                    .texture(handle.fetch_texture("inventory_slot").unwrap());
                if let Some(item) = &inventory.slots[i].item {
                    slot_element.sub_element_ex(||
                        EBuilder::new(border_width, border_width)
                            .shape(Shape::Rect(INVENTORY_SLOT_SIZE - border_width * 2.0, INVENTORY_SLOT_SIZE - border_width * 2.0))
                            .texture(item._type.fetch_texture(&mut handle.resource_manager))
                            .build()
                    );
                    if inventory.slots[i].quantity > 1 {
                        slot_element.sub_element_ex(||
                            EBuilder::new(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE - INVENTORY_SLOT_FONT_SIZE as f32)
                                .shape(Shape::Text(format!("{}", inventory.slots[i].quantity), INVENTORY_SLOT_FONT_SIZE, Allignment::Right))
                                .build()
                        );
                    }
                }
                element.sub_element_ex(|| slot_element.build());
            }
        }
    }
    state.gui.element(element.build());
}