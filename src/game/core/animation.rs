use hecs::Entity;
use rain::engine::{component::UVRect, core::RainHandle};

pub struct Animation {
    frames: Vec<AnimationFrame>,
    current_frame: usize,
    frame_progress: usize,
    repeat: bool,
}

pub struct AnimationFrame {
    uv_rect: UVRect,
    duration: usize,
}

impl Animation {
    pub fn new(frames: Vec<AnimationFrame>, repeat: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            frame_progress: 0,
            repeat,
        }
    }
}

impl AnimationFrame {
    pub fn new(uv_rect: UVRect, duration: usize) -> Self {
        Self { uv_rect, duration }
    }
}

pub fn system_manage_animations(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();

    for (e, (animation, uv_rect)) in handle.world.query_mut::<(&mut Animation, &mut UVRect)>() {
        let current_frame = &animation.frames[animation.current_frame];
        *uv_rect = current_frame.uv_rect;

        animation.frame_progress += 1;

        if animation.frame_progress >= current_frame.duration {
            animation.frame_progress = 0;
            animation.current_frame += 1;
        }

        if animation.current_frame >= animation.frames.len() {
            if animation.repeat {
                animation.current_frame = 0;
            } else {
                to_despawn.push(e);
            }
        }
    }

    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}