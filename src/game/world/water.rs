use hecs::Entity;
use rain::engine::component::Position2D;
use rain::engine::core::RainHandle;

use crate::State;
use crate::game::core::collision::{Collider, collect_water_colliders};


pub struct Swimmable;
pub struct Swimming;

pub fn system_swimming(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_swimming: Vec<Entity> = Vec::new();
    let mut to_remove_swimming: Vec<Entity> = Vec::new();

    for (e, (_, position, collider, swimming)) in handle.world.query::<(&Swimmable, &Position2D, &Collider, Option<&Swimming>)>().iter() {
        let colliders: Vec<Collider> = collect_water_colliders(state, position.0);

        let mut collided = false;
        let collider_center = collider.center();
        let new_collider = Collider::from_center(collider_center.x, collider_center.y, collider.width / 2.0, collider.height / 2.0);
        for water_collider in colliders {
            if new_collider.aabb_collision(&water_collider) {
                collided = true;
            }
        }
        match (collided, swimming.is_some()) {
            (true, false) => to_add_swimming.push(e),
            (false, true) => to_remove_swimming.push(e),
            _ => {}
        }
    }

    for e in to_add_swimming {
        handle.world.insert_one(e, Swimming).unwrap();
    }
    for e in to_remove_swimming {
        handle.world.remove_one::<Swimming>(e).unwrap();
    }
}