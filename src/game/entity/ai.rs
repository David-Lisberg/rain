use std::{collections::VecDeque, f32::consts::PI};

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::{Friction, Position2D, Velocity2D}, core::RainHandle};
use rand::RngExt;

use crate::{State, game::{core::{collision::Collider, physics::ADJACENT_I32}, entity::{damage::HitBox, enemy::Enemy, path::Path}, player::movement::Player, utility::timer::Timer, world::chunk::{ChunkPosition, position_to_chunk_position}}};

const ADJACENT: [Vec2; 8] = [
    Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0),
    Vec2::new(0.0, 0.5), Vec2::new(0.0, -0.5),
    Vec2::new(0.5, 0.5), Vec2::new(0.5, -0.5),
    Vec2::new(-0.5, 0.5), Vec2::new(-0.5, -0.5),
];
const EPSILON: f32 = 0.001;
const IDLE_TIME: i32 = 600;

pub struct Idle;
pub struct Tracking(Entity, Timer);
pub struct Attacking(Entity, Timer, bool);

#[derive(Clone)]
struct AStarNode {
    position: Vec2,
    parent: usize,
    f: f32,
    g: f32,
    h: f32,
}

impl AStarNode {
    fn default(position: Vec2, parent: usize) -> Self {
        Self { position, parent, f: 0.0, g: 0.0, h: 0.0 }
    }
}

pub fn system_enemy_idle(handle: &mut RainHandle, state: &mut State) {
    let mut to_track: Vec<Entity> = Vec::new();
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();
    
    let mut player: Option<(Entity, Position2D)> = None;
    for (e, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player = Some((e, position.clone()));
    }
    let (player_e, player_position) = player.unwrap();

    for (i, (e, (_, enemy, position, collider))) in handle.world.query::<(&Idle, &Enemy, &Position2D, &Collider)>().iter().enumerate() {
        if check_line_of_sight(state, position.0, player_position.0, collider, enemy.sight_range) {
            to_track.push(e);
        }
        if state.counter % IDLE_TIME == (i as i32 * 137) % IDLE_TIME {
            let radians = state.rng.random::<f32>() * 2.0 * PI;
            let direction = Vec2::from_angle(radians);
            let distance = state.rng.random::<f32>() * 4.0 + 5.0;
            let target = direction * distance + position.0;
            if let Some(path) = generate_path(state, position.0, target, collider) {
                to_add_path.push((e, path));
            }
        }
    }

    for e in to_track {
        handle.world.insert_one(e, Tracking(player_e, Timer::new(5.0))).unwrap();
        handle.world.remove_one::<Idle>(e).unwrap();
    }
    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
}

pub fn system_enemy_tracking(handle: &mut RainHandle, state: &mut State) {
    let mut to_idle: Vec<Entity> = Vec::new();
    let mut to_attack: Vec<(Entity, Entity)> = Vec::new();
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, tracking) in handle.world.query::<&Tracking>().iter() {
        let position = handle.world.get::<&Position2D>(tracking.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((tracking, enemy, position, collider)) = handle.world.query_one_mut::<(&mut Tracking, &Enemy, &Position2D, &Collider)>(e) {
            if check_line_of_sight(state, position.0, target_position, collider, enemy.tracking_range) {
                tracking.1.reset();
            } else {
                if tracking.1.step(handle.delta_time) {
                    to_idle.push(e);
                    continue;
                }
            }

            if (target_position - position.0).length() <= enemy.tracking_distance + 1.0 {
                to_attack.push((e, tracking.0));
                continue;
            }

            let direction = (position.0 - target_position).normalize();
            let tracking_position = target_position + direction * enemy.tracking_distance;
            if let Some(path) = generate_path(state, position.0, tracking_position, collider) {
                to_add_path.push((e, path));
            }
        }
    }

    for e in to_idle {
        handle.world.insert_one(e, Idle).unwrap();
        handle.world.remove_one::<Tracking>(e).unwrap();
    }
    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
    for (e, target_entity) in to_attack {
        handle.world.insert_one(e, Attacking(target_entity, Timer::new(1.0), false)).unwrap();
        handle.world.remove_one::<Tracking>(e).unwrap();
        let removed = handle.world.remove_one::<Path>(e).is_ok();
        if removed {
            if let Ok(mut velocity) = handle.world.get::<&mut Velocity2D>(e) {
                velocity.0 = Vec2::ZERO;
            }
        }
    }
}

pub fn system_enemy_attacking(handle: &mut RainHandle) {
    let mut to_idle: Vec<Entity> = Vec::new();
    let mut to_add_friction: Vec<(Entity, Friction)> = Vec::new();
    let mut to_add_hitbox: Vec<(Entity, HitBox)> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, attacking) in handle.world.query::<&Attacking>().iter() {
        let position = handle.world.get::<&Position2D>(attacking.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((attacking, enemy, position, velocity)) = handle.world.query_one_mut::<(&mut Attacking, &Enemy, &Position2D, &mut Velocity2D)>(e) {
            if !attacking.1.step(handle.delta_time) {
                continue;
            }
            if velocity.0.length() < 0.01 {
                if attacking.2 {
                    to_idle.push(e);
                } else {
                    let direction = (target_position - position.0).normalize();
                    velocity.0 = direction * enemy.attack_speed;
                    to_add_friction.push((e, Friction(5.0)));

                    let hitbox_offset = direction + position.0;
                    let hitbox = HitBox::new(
                        enemy.damage, Collider::from_center(hitbox_offset.x, hitbox_offset.y, 0.4, 0.4), vec![e], 1,
                    );
                    to_add_hitbox.push((e, hitbox));

                    attacking.2 = true;
                }
            }
        }
    }

    for e in to_idle {
        handle.world.insert_one(e, Idle).unwrap();
        handle.world.remove_one::<Attacking>(e).unwrap();
        let _ = handle.world.remove::<(Friction, HitBox)>(e).is_ok();
    }
    for (e, friction) in to_add_friction {
        handle.world.insert_one(e, friction).unwrap();
    }
    for (e, hitbox) in to_add_hitbox {
        handle.world.insert_one(e, hitbox).unwrap();
    }
}

fn check_line_of_sight(state: &mut State, start: Vec2, finish: Vec2, collider: &Collider, sight_range: f32) -> bool {
    if (finish - start).length() > sight_range {
        return false;
    }
    let object_colliders = fetch_object_colliders(state, start);

    let collider_center = collider.center();
    let new_collider = Collider::from_center(collider_center.x, collider_center.y, collider.width / 2.0, collider.height / 2.0);
    if line_of_sight_raycast(start, finish, Some(&new_collider), &object_colliders) {
        return true;
    }

    false
}

pub fn fetch_object_colliders(state: &mut State, position: Vec2) -> Vec<Collider> {
    let mut object_colliders: Vec<Collider> = Vec::new();
    let chunk_position = position_to_chunk_position(position.x, position.y);
    for adjacent in ADJACENT_I32 {
        let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
        if let Some(chunk) = state.chunks.get(&adjacent_position) {
            for object in &chunk.objects {
                if object.collidable {
                    object_colliders.push(object.collider.clone());
                }
            }
        }
    }
    object_colliders
}

fn generate_path(state: &mut State, start: Vec2, finish: Vec2, collider: &Collider) -> Option<Path> {
    let object_colliders = fetch_object_colliders(state, start);
    let positions = a_star(start, finish, collider, &object_colliders);

    if !positions.is_empty() {
        let path = Path::new(positions.into_iter().collect());
        return Some(path);
    }
    None
}

fn a_star(start: Vec2, finish: Vec2, collider: &Collider, other_colliders: &Vec<Collider>) -> VecDeque<Vec2> {
    let start = start.round();
    let finish = finish.round();

    if start.abs_diff_eq(finish, EPSILON) {
        return VecDeque::new();
    }

    let mut open_list: Vec<AStarNode> = Vec::new();
    let mut closed_list: Vec<AStarNode> = Vec::new();
    let mut final_node: Option<AStarNode> = None;
    open_list.push(AStarNode::default(start, 0));

    while !open_list.is_empty() {
        let (index, _) = open_list.iter()
            .enumerate()
            .min_by(|a, b| a.1.f.partial_cmp(&b.1.f).unwrap())
            .unwrap();
        let node = open_list.remove(index);

        for adjacent in ADJACENT {
            let mut successor = AStarNode::default(node.position + adjacent, closed_list.len());

            successor.g = node.g + adjacent.length();
            successor.h = (finish - successor.position).length();
            successor.f = successor.g + successor.h * 1.5;

            if successor.position.abs_diff_eq(finish, EPSILON) {
                final_node = Some(successor);
                break;
            }

            let successor_collider = Collider::from_center(successor.position.x as f32, successor.position.y as f32, collider.width, collider.height);
            if other_colliders.iter().any(|other| successor_collider.aabb_collision(other)) {
                continue;
            }

            if open_list.iter()
               .any(|other| other.position.abs_diff_eq(successor.position, EPSILON) && other.f < successor.f) ||
               closed_list.iter()
               .any(|other| other.position.abs_diff_eq(successor.position, EPSILON) && other.f < successor.f) {
                continue;
            }
            open_list.push(successor);
        }
        closed_list.push(node);
        if final_node.is_some() || open_list.len() >= 200 {
            break;
        }
    }

    if let Some(f) = final_node {
        a_star_node_to_path(&closed_list, f)
    } else {
        let closest = closed_list.iter()
            .min_by(|a, b| a.h.partial_cmp(&b.h).unwrap());
        if let Some(c) = closest {
            if c.parent != 0 {
                return a_star_node_to_path(&closed_list, c.clone());
            }
        }
        VecDeque::new()
    }
}

fn a_star_node_to_path(closed_list: &Vec<AStarNode>, final_node: AStarNode) -> VecDeque<Vec2> {
    let mut path: VecDeque<Vec2> = VecDeque::new();
    let mut current_index = final_node.parent;
    path.push_front(final_node.position);

    while current_index != 0 {
        let current = &closed_list[current_index];
        current_index = current.parent;
        path.push_front(current.position);
    }
    let start = &closed_list[0].position;
    path.push_front(*start);

    path
}

fn line_of_sight_raycast(start: Vec2, finish: Vec2, collider: Option<&Collider>, other_colliders: &Vec<Collider>) -> bool {
    if let Some(c) = collider {
        !other_colliders.iter().any(|other| c.aabb_collision_swept(other, &start, &finish))
    } else {
        !other_colliders.iter().any(|other| other.aabb_collision_ray(&start, &finish))
    }
}