use std::collections::HashMap;

use hecs::Entity;
use noise::Perlin;
use rain::engine::core::RainHandle;
use rand::Rng;

use crate::State;
pub struct Persistent;

pub fn system_reset_world(handle: &mut RainHandle, state: &mut State) {
    if state.to_reset {
        state.to_reset = false;

        let mut rng = rand::rng();
        let seed = rng.next_u32();
        let perlin = Perlin::new(seed);

        state.rng = rng;
        state.perlin = perlin;
        state.chunks = HashMap::new();
        state.enemy_count = 0;

        let to_despawn: Vec<Entity> = handle.world.query::<()>()
            .without::<&Persistent>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for e in to_despawn {
            handle.world.despawn(e).unwrap();
        }
    }
}