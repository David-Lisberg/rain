use hecs::Entity;
use rain::engine::core::RainHandle;

use crate::game::entity::damage::HitBox;

pub struct Projectile;

pub fn system_manage_projectiles(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();
    for (e, (_, hitbox)) in handle.world.query::<(&Projectile, Option<&HitBox>)>().iter() {
        if hitbox.is_none() {
            to_despawn.push(e);
        }
    }
    
    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}