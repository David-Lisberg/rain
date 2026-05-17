use std::collections::VecDeque;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::{Position2D, Velocity2D}, core::RainHandle};

use crate::{State, game::{core::{collision::Collider, physics::ADJACENT_I32}, entity::{enemy::Enemy, path::Path}, player::movement::Player, utility::timer::Timer, world::chunk::{ChunkPosition, position_to_chunk_position}}};

const ADJACENT: [Vec2; 8] = [
    Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0),
    Vec2::new(0.0, 0.5), Vec2::new(0.0, -0.5),
    Vec2::new(0.5, 0.5), Vec2::new(0.5, -0.5),
    Vec2::new(-0.5, 0.5), Vec2::new(-0.5, -0.5),
];
const EPSILON: f32 = 0.001;

pub struct Target(Entity);
pub struct TimerLoseTarget(Timer);

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

pub fn system_enemy_line_of_sight(handle: &mut RainHandle, state: &mut State) {
    let mut player: Option<(Entity, Position2D)> = None;
    for (e, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player = Some((e, position.clone()));
    }
    let (player_e, player_position) = player.unwrap();
    let mut to_target: Vec<Entity> = Vec::new();

    for (e, (_, position, collider)) in handle.world.query::<(&Enemy, &Position2D, &Collider)>().iter() {
        if (position.0 - player_position.0).length() <= 60.0 {
            let mut object_colliders: Vec<Collider> = Vec::new();
            let chunk_position = position_to_chunk_position(position.0.x, position.0.y);
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

            let collider_center = collider.center();
            let new_collider = Collider::from_center(collider_center.x, collider_center.y, collider.width / 2.0, collider.height / 2.0);
            if line_of_sight_raycast(position.0, player_position.0, Some(&new_collider), &object_colliders) {
                to_target.push(e);
            }
        } 
    }

    for e in to_target {
        handle.world.insert(e, (Target(player_e), TimerLoseTarget(Timer(5.0)))).unwrap();
    }
}

pub fn system_enemy_pathfinding(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();

    for (i, (e, (_, position, collider, target))) in handle.world.query::<(&Enemy, &Position2D, &Collider, &Target)>().iter().enumerate() {
        if state.counter % 60 != i as i32 {
            continue;
        }
        if let Ok(target_position) = handle.world.get::<&Position2D>(target.0) {
            let mut object_colliders: Vec<Collider> = Vec::new();
            let chunk_position = position_to_chunk_position(position.0.x, position.0.y);
            for adjacent in ADJACENT {
                let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.x as i32, chunk_position.y + adjacent.y as i32);
                if let Some(chunk) = state.chunks.get(&adjacent_position) {
                    for object in &chunk.objects {
                        if object.collidable {
                            object_colliders.push(object.collider.clone());
                        }
                    }
                }
            }
            let positions = a_star(position.0, target_position.0, collider, &object_colliders);
    
            if !positions.is_empty() {
                let path = Path::new(positions.into_iter().collect());
                to_add_path.push((e, path));
            }
        }
    }
    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
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

pub fn system_timer_lose_target(handle: &mut RainHandle) {
    let mut to_lose_target: Vec<Entity> = Vec::new();

    for (e, timer_lose_target) in handle.world.query_mut::<&mut TimerLoseTarget>() {
        if timer_lose_target.0.step(handle.delta_time) {
            to_lose_target.push(e);
        }
    }

    for e in to_lose_target {
        handle.world.remove::<(Target, TimerLoseTarget)>(e).unwrap();
        let removed = handle.world.remove_one::<Path>(e).is_ok();
        if removed {
            if let Ok(mut velocity) = handle.world.get::<&mut Velocity2D>(e) {
                velocity.0 = Vec2::ZERO;
            }
        }
    }
}