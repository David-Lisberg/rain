use std::{cmp::Ordering, collections::VecDeque};

use glam::{IVec2, Vec2};
use hecs::Entity;
use rain::engine::{component::Position2D, core::RainHandle};

use crate::{State, game::{entity::{enemy::Enemy, path::Path}, player::movement::Player}};

const ADJACENT: [IVec2; 8] = [
    IVec2::new(1, 0), IVec2::new(-1, 0),
    IVec2::new(0, 1), IVec2::new(0, -1),
    IVec2::new(1, 1), IVec2::new(1, -1),
    IVec2::new(-1, 1), IVec2::new(-1, -1),
];

struct AStarNode {
    position: IVec2,
    parent: usize,
    f: f32,
    g: f32,
    h: f32,
}

impl AStarNode {
    fn default(position: IVec2, parent: usize) -> Self {
        Self { position, parent, f: 0.0, g: 0.0, h: 0.0 }
    }
}

pub fn system_enemy_pathfinding(handle: &mut RainHandle, state: &mut State) {
    if state.counter % 60 != 0 {
        return;
    }
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();
    let mut player_position: Option<Position2D> = None;
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player_position = Some(position.clone())
    }
    let player_position = player_position.unwrap();

    for (e, (_, position)) in handle.world.query::<(&Enemy, &Position2D)>().iter() {
        let positions = a_star(position.0.as_ivec2(), player_position.0.as_ivec2());
        if !positions.is_empty() {
            let path = Path::new(positions.into_iter().collect());
            to_add_path.push((e, path));
        }
    }
    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
}

fn a_star(start: IVec2, finish: IVec2) -> VecDeque<IVec2> {
    if start == finish {
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

            successor.g = node.g + adjacent.as_vec2().length();
            successor.h = (finish - successor.position).as_vec2().length();
            successor.f = successor.g + successor.h * 1.1;

            if successor.position == finish {
                final_node = Some(successor);
                break;
            }

            if open_list.iter()
               .any(|other| other.position == successor.position && other.f < successor.f) ||
               closed_list.iter()
               .any(|other| other.position == successor.position && other.f < successor.f) {
                continue;
            }
            open_list.push(successor);
        }
        closed_list.push(node);
        if final_node.is_some() {
            break;
        }
    }
    
    if let Some(f) = final_node {
        a_star_node_to_path(&closed_list, f)
    } else {
        VecDeque::new()
    }
}

fn a_star_node_to_path(closed_list: &Vec<AStarNode>, final_node: AStarNode) -> VecDeque<IVec2> {
    let mut path: VecDeque<IVec2> = VecDeque::new();
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