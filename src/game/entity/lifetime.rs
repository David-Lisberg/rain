use hecs::Entity;
use rain::engine::core::RainHandle;

pub struct Lifetime(pub f32);

pub fn system_lifetime(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (e, lifetime) in handle.world.query_mut::<&mut Lifetime>() {
        if lifetime.0 <= 0.0 {
            to_despawn.push(e);
        } else {
            lifetime.0 -= handle.delta_time;
        }
    }

    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}