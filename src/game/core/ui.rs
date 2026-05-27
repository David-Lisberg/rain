use glam::Vec2;
use hecs::Entity;
use lgui::element::*;
use rain::engine::color::Color;
use rain::engine::component::*;
use rain::engine::core::RainHandle;

use crate::game::entity::damage::{Health, HealthBar};
use crate::game::player::inventory::{INVENTORY_SLOTS_HOTBAR, INVENTORY_SLOTS_INVENTORY, INVENTORY_SLOTS_WIDTH, Inventory};
use crate::game::player::movement::Player;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, State};

pub const INVENTORY_SLOT_SIZE: f32 = 54.0;
pub const INVENTORY_SLOT_HEIGHT: f32 = SCREEN_HEIGHT - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
pub const INVENTORY_SLOT_GAP: f32 = 10.0;
pub const INVENTORY_GAP: f32 = 250.0;
pub const INVENTORY_SLOT_FONT_SIZE: u32 = 15;
const HEALTH_BAR_GAP: f32 = 40.0;
const HEALTH_BAR_HEIGHT: f32 = INVENTORY_SLOT_HEIGHT - HEALTH_BAR_GAP;
const HEALTH_BAR_WIDTH: f32 = INVENTORY_SLOT_SIZE * 7.5;
const ENEMY_HEALTH_BAR_WIDTH: f32 = 25.0;

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
    let num_slots = INVENTORY_SLOTS_HOTBAR.len() as f32;
    let slots_width = num_slots * INVENTORY_SLOT_SIZE + (num_slots - 1.0) * INVENTORY_SLOT_GAP;
    let start = (SCREEN_WIDTH - slots_width) / 2.0; 
    let border_width = INVENTORY_SLOT_SIZE / 8.0;

    let mut element = EBuilder::new(0.0, 0.0)
        .shape(Shape::Rect(SCREEN_WIDTH, SCREEN_HEIGHT))
        .scale(Scale::NormalShift)
        .visible(false);

    for (_, (_, inventory, health)) in handle.world.query::<(&Player, &Inventory, &Health)>().iter() {

        for i in INVENTORY_SLOTS_HOTBAR {
            let x = start + i as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
            let mut slot_element = EBuilder::new(x, INVENTORY_SLOT_HEIGHT)
                .shape(Shape::Rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE))
                .texture(handle.fetch_texture("inventory_slot").unwrap());
            if inventory.selected.contains(&i) {
                slot_element.sub_element_ex(||
                    EBuilder::new(border_width / 2.0, border_width / 2.0)
                        .rect(INVENTORY_SLOT_SIZE - border_width, INVENTORY_SLOT_SIZE - border_width)
                        .texture(handle.fetch_texture("inventory_slot_selected").unwrap())
                        .build()    
                );
            }
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
                if inventory.selected.contains(&i) {
                    slot_element.sub_element_ex(||
                        EBuilder::new(border_width / 2.0, border_width / 2.0)
                            .rect(INVENTORY_SLOT_SIZE - border_width, INVENTORY_SLOT_SIZE - border_width)
                            .texture(handle.fetch_texture("inventory_slot_selected").unwrap())
                            .build()    
                    );
                }
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

            let num_recipes = inventory.available_recipes.len() as f32;
            let recipes_width = num_recipes * INVENTORY_SLOT_SIZE + (num_recipes - 1.0) * INVENTORY_SLOT_GAP;
            let start_recipe = (SCREEN_WIDTH - recipes_width) / 2.0;
            let y = INVENTORY_SLOT_HEIGHT - INVENTORY_GAP - (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP) * 3.2;
            for (i, recipe) in inventory.available_recipes.iter().enumerate() {
                let x = start_recipe + (i % INVENTORY_SLOTS_WIDTH) as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
                let mut recipe_element = EBuilder::new(x, y)
                    .rect(INVENTORY_SLOT_SIZE, INVENTORY_SLOT_SIZE)
                    .texture(handle.fetch_texture("inventory_slot").unwrap());
                recipe_element.sub_element_ex(|| EBuilder::new(border_width, border_width)
                    .rect(INVENTORY_SLOT_SIZE - border_width * 2.0, INVENTORY_SLOT_SIZE - border_width * 2.0)
                    .texture(recipe.output.0.fetch_texture(&mut handle.resource_manager))
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
        } else {
            let x = start + inventory.selected_hotbar as f32 * (INVENTORY_SLOT_SIZE + INVENTORY_SLOT_GAP);
            element.sub_element_ex(||
                EBuilder::new(x + border_width / 2.0, INVENTORY_SLOT_HEIGHT + border_width / 2.0)
                    .rect(INVENTORY_SLOT_SIZE - border_width, INVENTORY_SLOT_SIZE - border_width)
                    .texture(handle.fetch_texture("inventory_slot_selected").unwrap())
                    .build()    
            );
        }

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
    state.gui.element(element.build());
}