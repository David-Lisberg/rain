use rain::engine::{component::*, core::RainHandle};

use crate::{State, game::{core::collision::*, player::{inventory::Inventory, item::*, movement::Player}, world::object::{ObjectType, destroy_object, reload_object_mesh}}};

pub fn item_pickup(handle: &mut RainHandle, state: &mut State) {
    let mut object_changed = false;
    let query = handle.world.query_mut::<(&Player, &Position2D, &Direction, &mut Inventory)>();
    for (_, (_, position, direction, inventory)) in query {
        let collider_position = position.0 + direction.0;
        let collider = Collider::from_center(collider_position.x, collider_position.y, 1.0, 1.0);
        if let Some(object) = check_collision_with_object(state, &collider) {
            match object._type {
                ObjectType::Twig => {
                    if destroy_object(state, &object) {
                        object_changed = true;
                        inventory.add_item(Item::new(ItemType::Twig), 1);
                    }
                }
                _ => {}
            }
        }
    }

    if object_changed {
        reload_object_mesh(handle, state);
    }
}