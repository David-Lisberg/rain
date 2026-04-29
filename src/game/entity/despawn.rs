use hecs::Entity;
use rain::engine::core::RainHandle;

use crate::game::utility::timer::Timer;

pub struct TimerDespawn(pub Timer);

pub fn system_timer_despawn(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (e, timer_despawn) in handle.world.query_mut::<&mut TimerDespawn>() {
        if timer_despawn.0.step(handle.delta_time) {
            to_despawn.push(e);
        }
    }

    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}