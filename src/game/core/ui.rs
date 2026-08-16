use glam::Vec2;
use hecs::Entity;
use lgui::element::*;
use rain::engine::color::Color;
use rain::engine::component::*;
use rain::engine::core::RainHandle;

use crate::game::entity::damage::{Health, HealthBar};
use crate::game::player::inventory::{CraftHover, Inventory, InventoryHover, InventorySelection, PlayerInventory};
use crate::game::player::item::Item;
use crate::game::player::movement::Player;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State};

pub const INVENTORY_SLOT_SIZE: f32 = 54.0;
pub const INVENTORY_SLOT_HEIGHT: f32 = SCREEN_HEIGHT - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
pub const INVENTORY_SLOT_GAP: f32 = 10.0;
pub const INVENTORY_GAP: f32 = 200.0;
pub const INVENTORY_SLOT_FONT_SIZE: u32 = 15;
const HEALTH_BAR_GAP: f32 = 40.0;
const HEALTH_BAR_HEIGHT: f32 = INVENTORY_SLOT_HEIGHT - HEALTH_BAR_GAP;
const HEALTH_BAR_WIDTH: f32 = INVENTORY_SLOT_SIZE * 7.5;
const ENEMY_HEALTH_BAR_WIDTH: f32 = 25.0;
const MOUSE_OFFSET: f32 = 12.0;
const BLURB_FONT_SIZE: u32 = 12;

pub fn render_ui(handle: &mut RainHandle, state: &mut State) {
    let current_width = handle.renderer.config.width as f32;
    let current_height = handle.renderer.config.height as f32;

    state.gui.begin(current_width, current_height);

    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        state.gui.element(EBuilder::new(10.0, 10.0).shape(Shape::Text(format!("{}, {}", position.0.x, position.0.y), 20, Allignment::Left)).build());
    }
    render_health_bar(handle, state);
    render_inventory(handle, state);

    state.gui.finish(handle);
}

fn render_health_bar(handle: &mut RainHandle, state: &mut State) {
    let mut health_bars: Vec<(Entity, bool, f32)> = Vec::new();

    for (_, health_bar) in handle.world.query::<&HealthBar>().iter() {
        health_bars.push((health_bar.0, health_bar.1.finished(), health_bar.2));
    }
    for (parent, hide, health_percent) in health_bars {
        if hide {
            continue;
        }
        if let Ok(mut q) = handle.world.query_one::<(&Position2D, &Scale2D)>(parent) {
            if let Some((position, scale)) = q.get() {
                let world_position = Vec2::new(position.0.x, position.0.y + scale.0.y * 0.6);
                let screen_position = handle.world_position_to_screen_position(world_position);

                let background_color = Color::from_f32(0.0, 0.0, 0.0, 0.7);
                state.gui.element(EBuilder::new(screen_position.x - ENEMY_HEALTH_BAR_WIDTH / 2.0, screen_position.y)
                    .shape(Shape::Rect(ENEMY_HEALTH_BAR_WIDTH, ENEMY_HEALTH_BAR_WIDTH / 10.0))
                    .color(background_color.into())
                    .scale(Scale::None)
                    .build()
                );
                state.gui.element(EBuilder::new(screen_position.x - ENEMY_HEALTH_BAR_WIDTH / 2.0, screen_position.y)
                    .shape(Shape::Rect(ENEMY_HEALTH_BAR_WIDTH * health_percent, ENEMY_HEALTH_BAR_WIDTH / 10.0))
                    .color(Color::RED.into())
                    .scale(Scale::None)
                    .build()
                );
            }
        }
    }
}

fn render_inventory(handle: &mut RainHandle, state: &mut State) {
    let mut y_pointer = INVENTORY_SLOT_HEIGHT;
    let border_width = INVENTORY_SLOT_SIZE / 8.0;
    let mut element = EBuilder::new(0.0, 0.0)
        .rect(SCREEN_WIDTH, SCREEN_HEIGHT)
        .scale(Scale::NormalShift)
        .visible(false);

    let mut player_info: Option<(Entity, PlayerInventory, Option<InventoryHover>, Option<CraftHover>)> = None;
    for (e, (_, p, i_hover, c_hover)) in handle.world.query::<(
        &Player, &PlayerInventory, Option<&InventoryHover>, Option<&CraftHover>
    )>().iter() {
        player_info = Some((e, p.clone(), i_hover.cloned(), c_hover.cloned()));
    }
    let (player_entity, player_inventory, inventory_hover, craft_hover) = player_info.unwrap();

    for panel in state.inventory_screen.panels.iter() {
        y_pointer -= panel.gap;
        if let Some(inventory) = handle.world.query_one::<&Inventory>(panel.inventory).unwrap().get() {
            let slots_width = panel.ui.columns as f32 * panel.ui.slot_size + (panel.ui.columns - 1) as f32 * panel.ui.column_gap;
            let x_pointer = (SCREEN_WIDTH - slots_width) / 2.0;

            let range = panel.slots.clone().unwrap();
            let slots = 0..(range.end - range.start);
            for i in slots {
                let mut slot_element = EBuilder::new(
                    x_pointer + (i as i32 % panel.ui.columns) as f32 * (panel.ui.slot_size + panel.ui.column_gap), 
                    y_pointer - (i as i32 / panel.ui.columns) as f32 * (panel.ui.slot_size + panel.ui.row_gap)
                ).rect(panel.ui.slot_size, panel.ui.slot_size)
                    .texture(handle.fetch_texture(panel.ui.slot_texture).unwrap());
                if let Some(item) = &inventory.slots[i + range.start].item {
                    let item_data = state.item_registry.get(&item._type).unwrap();
                    slot_element.sub_element_ex(||
                        EBuilder::new(border_width, border_width)
                            .rect(INVENTORY_SLOT_SIZE - border_width * 2.0, INVENTORY_SLOT_SIZE - border_width * 2.0)
                            .texture(handle.resource_manager.fetch_texture(&item_data.texture).unwrap())
                            .build()
                    );
                    if inventory.slots[i + range.start].quantity > 1 {
                        slot_element.sub_element_ex(||
                            EBuilder::new(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE - INVENTORY_SLOT_FONT_SIZE as f32)
                                .shape(Shape::Text(format!("{}", inventory.slots[i + range.start].quantity), INVENTORY_SLOT_FONT_SIZE, Allignment::Right))
                                .build()
                        );
                    }
                }
                if state.inventory_screen.selection.contains(&InventorySelection::new(panel.inventory, i + range.start)) ||
                   (panel.inventory == player_entity && (i + range.start) == player_inventory.selected_hotbar && !player_inventory.open) {
                    slot_element.sub_element_ex(||
                        EBuilder::new(border_width / 2.0, border_width / 2.0)
                            .rect(INVENTORY_SLOT_SIZE - border_width, INVENTORY_SLOT_SIZE - border_width)
                            .texture(handle.fetch_texture("inventory_slot_selected").unwrap())
                            .build()    
                    );
                }
                element.sub_element_ex(|| slot_element.build());
            }
        }
        y_pointer -= panel.ui.rows as f32 * (panel.ui.slot_size + panel.ui.row_gap);
    }

    if player_inventory.display_recipes {
        let panel = state.inventory_screen.panels.first().unwrap();
        let num_recipes = player_inventory.available_recipes.len() as f32;
        let recipes_width = num_recipes * panel.ui.slot_size + (num_recipes - 1.0) * panel.ui.column_gap;
        let x_pointer = (SCREEN_WIDTH - recipes_width) / 2.0;
        y_pointer -= panel.ui.row_gap; 

        for (i, recipe) in player_inventory.available_recipes.iter().enumerate() {
            let item_data = state.item_registry.get(&recipe.output.0).unwrap();
            let mut recipe_element = EBuilder::new(x_pointer + i as f32 * (panel.ui.slot_size + panel.ui.column_gap), y_pointer)
                .rect(panel.ui.slot_size, panel.ui.slot_size)
                .texture(handle.fetch_texture("inventory_slot").unwrap());
            recipe_element.sub_element_ex(|| EBuilder::new(border_width, border_width)
                .rect(panel.ui.slot_size - border_width * 2.0, panel.ui.slot_size - border_width * 2.0)
                .texture(handle.resource_manager.fetch_texture(&item_data.texture).unwrap())
                .build()
            );
            if recipe.output.1 > 1 {
                recipe_element.sub_element_ex(||
                    EBuilder::new(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE - INVENTORY_SLOT_FONT_SIZE as f32)
                        .shape(Shape::Text(format!("{}", recipe.output.1), INVENTORY_SLOT_FONT_SIZE, Allignment::Right))
                        .build()
                );
            }
            element.sub_element_ex(|| recipe_element.build());
        }
    }

    if let Some(health) = handle.world.query_one::<&Health>(player_entity).unwrap().get() {
        element.sub_element_ex(||
            EBuilder::new((SCREEN_WIDTH - HEALTH_BAR_WIDTH) / 2.0, HEALTH_BAR_HEIGHT)
                .rect(HEALTH_BAR_WIDTH, HEALTH_BAR_WIDTH / 24.0)
                .texture(handle.fetch_texture("health_bar_background").unwrap())
                .build()
        );
        element.sub_element_ex(||
            EBuilder::new((SCREEN_WIDTH - HEALTH_BAR_WIDTH * 188.0 / 192.0) / 2.0, HEALTH_BAR_HEIGHT + HEALTH_BAR_WIDTH / 96.0)
                .rect(HEALTH_BAR_WIDTH * 188.0 / 192.0 * health.current / health.max, HEALTH_BAR_WIDTH / 48.0)
                .color(Color::RED.into())
                .build()
        );
        element.sub_element_ex(||
            EBuilder::new((SCREEN_WIDTH - HEALTH_BAR_WIDTH) / 2.0, HEALTH_BAR_HEIGHT)
                .rect(HEALTH_BAR_WIDTH, HEALTH_BAR_WIDTH / 24.0)
                .texture(handle.fetch_texture("health_bar_frame").unwrap())
                .build()
        );
    }

    state.gui.element_immediate(handle, element.build());

    let mouse_position = handle.mouse_position();
    if let Some(i_hover) = inventory_hover {
        let mut item: Option<Item> = None;
        if let Some(inventory) = handle.world.query_one::<&Inventory>(i_hover.0.inventory).unwrap().get() {
            item = inventory.slots[i_hover.0.slot].item.clone();
        }
        if let Some(i) = item {
            let item_data = state.item_registry.get(&i._type).unwrap();
            let width = handle.measure_text(&item_data.name, BLURB_FONT_SIZE);
            handle.draw_rectangle((mouse_position.x + MOUSE_OFFSET, mouse_position.y + MOUSE_OFFSET, width, BLURB_FONT_SIZE as f32), Color::BLACK);
            handle.draw_text(mouse_position.x + MOUSE_OFFSET, mouse_position.y + MOUSE_OFFSET, &item_data.name, BLURB_FONT_SIZE, Color::WHITE);
        }
    }
    if let Some(c_hover) = craft_hover {
        if let Some(recipe) = player_inventory.available_recipes.get(c_hover.0) {
            let item_data = state.item_registry.get(&recipe.output.0).unwrap();
            let mut strings = vec![format!("Craft {}x {}", recipe.output.1, item_data.name), "Uses:".to_string()];
            let width = handle.measure_text(&strings[0], BLURB_FONT_SIZE);

            for input in &recipe.input {
                let item_data = state.item_registry.get(&input.0).unwrap();
                strings.push(format!("{}x {}", input.1, item_data.name));
            }

            let height = BLURB_FONT_SIZE as f32 * 1.3 * strings.len() as f32;
            handle.draw_rectangle((mouse_position.x + MOUSE_OFFSET, mouse_position.y + MOUSE_OFFSET, width, height), Color::BLACK);

            for (i, string) in strings.iter().enumerate() {
                handle.draw_text(
                    mouse_position.x + MOUSE_OFFSET, mouse_position.y + MOUSE_OFFSET + BLURB_FONT_SIZE as f32 * 1.3 * i as f32, 
                    string, BLURB_FONT_SIZE, Color::WHITE
                );
            }
        }
    }
}