use hecs::Entity;
use rain::engine::{component::{DepthZ, Position2D, Scale2D}, core::RainHandle};

struct DepthDefault(f32);
pub const DEPTH_SCALE: f32 = 0.0001;

pub fn system_apply_depth_sort(handle: &mut RainHandle) {
    let mut to_add_depth_default: Vec<(Entity, f32)> = Vec::new();

    for (e, (position, scale, depth, default_depth)) in handle.world.query_mut::<(
        &Position2D, Option<&Scale2D>, &mut DepthZ, Option<&DepthDefault>
    )>() {
        let feet_position = match scale {
            Some(s) => position.0.y - s.0.y / 2.0,
            None => position.0.y - 0.5,
        };
        if let Some(default) = default_depth {
            depth.0 = default.0 + feet_position * DEPTH_SCALE;
        } else {
            to_add_depth_default.push((e, depth.0));
            depth.0 += feet_position * DEPTH_SCALE;
        }
        println!("{}", depth.0);
    }

    for (e, depth) in to_add_depth_default {
        handle.world.insert_one(e, DepthDefault(depth)).unwrap();
    }
}