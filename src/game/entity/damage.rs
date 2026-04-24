use hecs::Entity;
use rain::engine::core::RainHandle;

use crate::game::core::collision::Collider;

pub struct HurtBox(pub Collider);
#[derive(Clone)]
pub struct HitBox {
    pub damage: f32,
    pub collider: Collider,
    pub safe: Vec<Entity>,
}

pub struct Health(pub f32);

pub fn system_hitbox_hurtbox_collision(handle: &mut RainHandle) {
    let mut hitboxes: Vec<(Entity, HitBox)> = Vec::new();
    let mut to_remove: Vec<Entity> = Vec::new();
    for (e, hitbox) in handle.world.query::<&HitBox>().iter() {
        hitboxes.push((e, hitbox.clone()));
    }
    for (e, (hurtbox, health)) in handle.world.query_mut::<(&HurtBox, &mut Health)>() {
        for (other_e, hitbox) in hitboxes.iter() {
            if e == *other_e || hitbox.safe.contains(&e) {
                continue;
            }
            if hitbox.collider.aabb_collision(&hurtbox.0) {
                health.0 -= hitbox.damage;
                if health.0 <= 0.0 {
                    to_remove.push(e);
                    break;
                }
            }
        }
    }
    for e in to_remove {
        handle.world.despawn(e).unwrap();
    }
}