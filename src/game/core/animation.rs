use hecs::Entity;
use rain::engine::component::{Flip, Position2D};
use rain::engine::core::RainHandle;
use rain::engine::animation::{Animation, AnimationEvent, AnimationPool};

use crate::game::core::collision::Collider;
use crate::game::entity::damage::HitBox;
use crate::game::player::action::PlayerAttacking;
use crate::game::player::input::Lock;
use crate::game::player::inventory::Inventory;
use crate::game::player::item::ItemCategory;

#[derive(Clone)]
struct ActiveEvents {
    entity: Entity,
    events: Vec<AnimationEvent>,
    frames_left: usize,
}

impl ActiveEvents {
    pub fn new(entity: Entity, events: Vec<AnimationEvent>, frames_left: usize) -> Self {
        Self { entity, events, frames_left, }
    }
}

pub fn system_manage_animation_events(handle: &mut RainHandle) {
    let mut to_add_event: Vec<ActiveEvents> = Vec::new();
    let mut to_add_lock: Vec<Entity> = Vec::new();
    let mut to_add_hitbox: Vec<(Entity, HitBox)> = Vec::new();
    let mut to_add_component: Vec<(Entity, String)> = Vec::new();
    let mut to_remove_component: Vec<(Entity, String)> = Vec::new();

    let mut animation_ids: Vec<(Entity, String, Option<usize>)> = Vec::new();
    for (e, animation) in handle.world.query::<&Animation>().iter() {
        animation_ids.push((e, animation.name.clone(), None));
    }
    for (e, animation_pool) in handle.world.query::<&AnimationPool>().iter() {
        for (i, animation) in animation_pool.animations.iter() {
            animation_ids.push((e, animation.name.clone(), Some(*i)));
        }
    }
    for (e, id, index) in animation_ids {
        let animation_data = handle.fetch_animation(&id).unwrap();
        let animation = if let Some(i) = index {
            if let Ok(mut q) = handle.world.query_one::<&AnimationPool>(e) {
                q.get()
                    .and_then(|pool| pool.animations.get(&i))
                    .cloned()
            } else {
                None
            }
        } else {
            if let Ok(mut q) = handle.world.query_one::<&Animation>(e) {
                q.get().cloned()
            } else {
                None
            }
        };
        if let Some(a) = animation {
            let current_frame = &animation_data.frames[a.current_frame];
            let mut events: Vec<AnimationEvent> = Vec::new();
            if a.current_frame == 0 && a.frame_progress == 0 {
                if let Some(start) = &animation_data.start {
                    if let Some(other_events) = &start.events {
                        events.extend(other_events.clone());
                        let active_event = ActiveEvents::new(e, other_events.clone(), 1);
                        to_add_event.push(active_event);
                    }
                }
            }
            if let Some(animation_events) = &current_frame.events {
                if a.frame_progress == 0 {
                    events.extend(animation_events.clone());
                    let active_event = ActiveEvents::new(e, animation_events.clone(), current_frame.duration);
                    to_add_event.push(active_event);
                }
            }
            if a.frame_progress == current_frame.duration - 1 && a.current_frame == animation_data.frames.len() - 1 {
                if let Some(finish) = &animation_data.finish {
                    events.extend(finish.clone());
                    let active_event = ActiveEvents::new(e, finish.clone(), 1);
                    to_add_event.push(active_event);
                }
            }

            for event in events {
                match event {
                    AnimationEvent::HitBox(collider) => {
                        let hitbox_collider = Collider::from_center(collider[0], collider[1], collider[2], collider[3]);
                        let hitbox = HitBox::new(1.0, hitbox_collider, vec![e], 1);
                        to_add_hitbox.push((e, hitbox));
                    }
                    AnimationEvent::LockInput => to_add_lock.push(e),
                    AnimationEvent::AddComponent(component) => to_add_component.push((e, component.to_string())),
                    AnimationEvent::RemoveComponent(component) => to_remove_component.push((e, component.to_string())),
                }
            }
        }
    }

    for (e, component) in to_add_component {
        match component.as_str() {
            "Attacking" => handle.world.insert_one(e, PlayerAttacking).unwrap(),
            _ => {}
        }
    }
    for (e, component) in to_remove_component {
        match component.as_str() {
            "Attacking" => { handle.world.remove_one::<PlayerAttacking>(e).unwrap(); }
            _ => {}
        }
    }
    for active_event in to_add_event {
        handle.world.insert_one(active_event.entity, active_event).unwrap();
    }
    for (e, mut hitbox) in to_add_hitbox {
        if let Ok(mut q) = handle.world.query_one::<(Option<&Position2D>, Option<&Inventory>, Option<&Flip>)>(e) {
            let (position, inventory, flip) = q.get().unwrap();
            if let Some(f) = flip {
                if f.0 {
                    hitbox.collider.x *= -1.0;
                }
                if f.1 {
                    hitbox.collider.y *= -1.0;
                }
            }
            if let Some(p) = position {
                hitbox.collider.x += p.0.x;
                hitbox.collider.y += p.0.y;
            }
            if let Some(i) = inventory {
                let damage = if let Some(item) = &i.slots[i.selected_hotbar].item {
                    match item.category {
                        ItemCategory::Tool(_, _, _, d) => d,
                        _ => 1.0,
                    }
                } else {
                    1.0
                };
                hitbox.damage *= damage;
            }
        }
        handle.world.insert_one(e, hitbox).unwrap();
    }
    for e in to_add_lock {
        handle.world.insert_one(e, Lock).unwrap();
    }

    let mut to_remove_event: Vec<Entity> = Vec::new();

    for (e, active_events) in handle.world.query_mut::<&mut ActiveEvents>() {
        if active_events.frames_left <= 0 {
            to_remove_event.push(e);
            break;
        }

        active_events.frames_left -= 1;
    }

    for e in to_remove_event {
        let active_event = handle.world.remove_one::<ActiveEvents>(e).unwrap();
        for event in &active_event.events {
            match event {
                AnimationEvent::HitBox(_) => {
                    let _ = handle.world.remove_one::<HitBox>(active_event.entity).is_ok();
                }
                AnimationEvent::LockInput => {
                    handle.world.remove_one::<Lock>(active_event.entity).unwrap();
                }
                _ => {}
            }
        }
    }
}