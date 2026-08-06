use std::{collections::VecDeque, f32::consts::PI};

use glam::Vec2;
use hecs::Entity;
use rain::engine::component::*; 
use rain::engine::core::RainHandle;
use rand::RngExt;

use crate::State;
use crate::game::core::collision::Collider;
use crate::game::core::physics::ADJACENT_I32;
use crate::game::entity::damage::HitBox;
use crate::game::entity::enemy::{Enemy, Resource};
use crate::game::entity::path::Path;
use crate::game::entity::projectile::{ProjectileSpawn, spawn_projectile};
use crate::game::entity::transition::{TransitionCondition, TransitionState, TransitionStateContext};
use crate::game::player::movement::Player;
use crate::game::utility::timer::Timer;
use crate::game::world::chunk::{ChunkPosition, position_to_chunk_position};


const ADJACENT: [Vec2; 8] = [
    Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0),
    Vec2::new(0.0, 0.5), Vec2::new(0.0, -0.5),
    Vec2::new(0.5, 0.5), Vec2::new(0.5, -0.5),
    Vec2::new(-0.5, 0.5), Vec2::new(-0.5, -0.5),
];
const EPSILON: f32 = 0.001;

pub struct Idle;
pub struct Tracking(Entity, Timer);
pub struct Digging(Timer);
pub struct AttackingDash(Entity, Timer, bool);
pub struct AttackingProjectile(Entity, Timer);
pub struct Escaping(Entity, Timer);

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

pub fn system_enemy_ai(handle: &mut RainHandle, state: &mut State) {
    system_enemy_idle(handle, state);
    system_enemy_tracking(handle, state);
    system_enemy_digging(handle, state);
    system_enemy_attacking_dash(handle, state);
    system_enemy_attacking_projectile(handle, state);
    system_enemy_escaping(handle, state);
}

fn add_transition_state(handle: &mut RainHandle, entity: Entity, transition_state: TransitionState, context: TransitionStateContext) {
    match transition_state {
        TransitionState::AttackingDash => handle.world.insert_one(entity, AttackingDash(context.target, Timer::new(1.0), false)).unwrap(),
        TransitionState::AttackingProjectile => handle.world.insert_one(entity, AttackingProjectile(context.target, Timer::new(1.0))).unwrap(),
        TransitionState::Digging => handle.world.insert_one(entity, Digging(Timer::new(3.0))).unwrap(),
        TransitionState::Escaping => handle.world.insert_one(entity, Escaping(context.target, Timer::new(3.0))).unwrap(),
        TransitionState::Idle => handle.world.insert_one(entity, Idle).unwrap(),
        TransitionState::Tracking => handle.world.insert_one(entity, Tracking(context.target, Timer::new(4.0))).unwrap(),
    }
}

pub fn system_enemy_idle(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState)> = Vec::new();
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();
    
    let mut player: Option<(Entity, Position2D)> = None;
    for (e, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player = Some((e, position.clone()));
    }
    let (player_e, player_position) = player.unwrap();

    for (i, (e, (_, enemy, position, collider, resource))) in handle.world.query::<(
        &Idle, &Enemy, &Position2D, &Collider, Option<&Resource>
    )>().iter().enumerate() {
        let enemy_data = state.enemy_registry.get(&enemy.0).unwrap().clone();
        let do_idle_behavior = state.counter % enemy_data.idle_interval == (i as i32 * 137) % enemy_data.idle_interval;
        if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::Idle).clone() {
            for (transition_state, conditions) in transition_graph {
                let mut success = true;
                for condition in conditions {
                    match condition {
                        TransitionCondition::Actionable => success &= do_idle_behavior,
                        TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                        TransitionCondition::LineOfSight => success &= check_line_of_sight(state, position.0, player_position.0, collider, enemy_data.sight_range),
                        TransitionCondition::NotMaxResource => match resource {
                            Some(r) => success &= r.current < r.max.unwrap_or(0),
                            _ => {}
                        }
                        _ => {}
                    }
                }
                if success {
                    to_add_transition_state.push((e, transition_state.clone()));
                    break;
                }
            }
        }
        if do_idle_behavior {
            let target = generate_random_target(state, position.0, 4.0, 5.0);
            if let Some(path) = generate_path(state, position.0, target, collider) {
                to_add_path.push((e, path));
            }
        }
    }

    for (e, transition_state) in to_add_transition_state {
        handle.world.remove_one::<Idle>(e).unwrap();
        remove_path(handle, e);
        add_transition_state(handle, e, transition_state, TransitionStateContext { target: player_e });
    }
    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
}

fn generate_random_target(state: &mut State, start: Vec2, range: f32, min_distance: f32) -> Vec2 {
    let radians = state.rng.random::<f32>() * 2.0 * PI;
    let direction = Vec2::from_angle(radians);
    let distance = state.rng.random::<f32>() * range + min_distance;
    direction * distance + start
}

pub fn system_enemy_digging(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState)> = Vec::new();

    let mut player: Option<(Entity, Position2D)> = None;
    for (e, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        player = Some((e, position.clone()));
    }
    let (player_e, player_position) = player.unwrap();

    for (e, (digging, enemy, position, collider, resource)) in handle.world.query_mut::<(
        &mut Digging, &Enemy, &Position2D, &Collider, &mut Resource
    )>() {
        let enemy_data = state.enemy_registry.get(&enemy.0).unwrap().clone();
        let actionable = digging.0.step(handle.delta_time);
        if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::Digging).clone() {
            for (transition_state, conditions) in transition_graph {
                let mut success = true;
                for condition in conditions {
                    match condition {
                        TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                        TransitionCondition::LineOfSight => success &= check_line_of_sight(state, position.0, player_position.0, collider, enemy_data.sight_range),
                        TransitionCondition::NotMaxResource => success &= resource.current < resource.max.unwrap_or(0),
                        TransitionCondition::Actionable => success &= actionable,
                        _ => {}
                    }
                }
                if success {
                    to_add_transition_state.push((e, transition_state.clone()));
                    break;
                }
            }
        }
        if actionable {
            resource.current += 1;
            digging.0.reset();
        }
    }

    for (e, transition_state) in to_add_transition_state {
        handle.world.remove_one::<Digging>(e).unwrap();
        add_transition_state(handle, e, transition_state, TransitionStateContext { target: player_e });
    }
}

pub fn system_enemy_tracking(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState, TransitionStateContext)> = Vec::new();
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, tracking) in handle.world.query::<&Tracking>().iter() {
        let position = handle.world.get::<&Position2D>(tracking.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((tracking, enemy, position, collider)) = handle.world.query_one_mut::<(
            &mut Tracking, &Enemy, &Position2D, &Collider
        )>(e) {
            let enemy_data = state.enemy_registry.get(&enemy.0).unwrap().clone();
            let mut actionable = false;
            if check_line_of_sight(state, position.0, target_position, collider, enemy_data.tracking_range) {
                tracking.1.reset();
            } else {
                if tracking.1.step(handle.delta_time) {
                    actionable = true;
                }
            }
            if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::Tracking).clone() {
                for (transition_state, conditions) in transition_graph {
                    let mut success = true;
                    for condition in conditions {
                        match condition {
                            TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                            TransitionCondition::LineOfSight => success &= check_line_of_sight(state, position.0, target_position, collider, enemy_data.sight_range),
                            TransitionCondition::Actionable => success &= actionable,
                            TransitionCondition::InAttackRange => success &= (target_position - position.0).length() <= enemy_data.tracking_distance + 1.0,
                            _ => {}
                        }
                    }
                    if success {
                        to_add_transition_state.push((e, transition_state.clone(), TransitionStateContext{ target: tracking.0 }));
                        break;
                    }
                }
            }

            let direction = (position.0 - target_position).normalize();
            let tracking_position = target_position + direction * enemy_data.tracking_distance;
            if let Some(path) = generate_path(state, position.0, tracking_position, collider) {
                to_add_path.push((e, path));
            }
        }
    }

    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
    for (e, transition_state, context) in to_add_transition_state {
        handle.world.remove_one::<Tracking>(e).unwrap();
        remove_path(handle, e);
        add_transition_state(handle, e, transition_state, context);
    }
}

fn remove_path(handle: &mut RainHandle, entity: Entity) {
    let removed = handle.world.remove_one::<Path>(entity).is_ok();
    if removed {
        if let Ok(mut velocity) = handle.world.get::<&mut Velocity2D>(entity) {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn system_enemy_attacking_dash(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState, TransitionStateContext)> = Vec::new();
    let mut to_add_friction: Vec<(Entity, Friction)> = Vec::new();
    let mut to_add_hitbox: Vec<(Entity, HitBox)> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, attacking) in handle.world.query::<&AttackingDash>().iter() {
        let position = handle.world.get::<&Position2D>(attacking.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((attacking, enemy, position, velocity)) = handle.world.query_one_mut::<(
            &mut AttackingDash, &Enemy, &Position2D, &mut Velocity2D
        )>(e) {
            if !attacking.1.step(handle.delta_time) {
                continue;
            }
            let enemy_data = state.enemy_registry.get(&enemy.0).unwrap();
            let mut actionable = false;
            if velocity.0.length() < 0.01 {
                if attacking.2 {
                    actionable = true;
                } else {
                    let direction = (target_position - position.0).normalize();
                    velocity.0 = direction * enemy_data.attack_speed;
                    to_add_friction.push((e, Friction(5.0)));

                    let hitbox_offset = direction + position.0;
                    let hitbox = HitBox::new(
                        enemy_data.damage, Collider::from_center(hitbox_offset.x, hitbox_offset.y, 0.4, 0.4), vec![e], 1,
                    );
                    to_add_hitbox.push((e, hitbox));

                    attacking.2 = true;
                }
            }
            if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::AttackingDash).clone() {
                for (transition_state, conditions) in transition_graph {
                    let mut success = true;
                    for condition in conditions {
                        match condition {
                            TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                            TransitionCondition::Actionable => success &= actionable,
                            _ => {}
                        }
                    }
                    if success {
                        to_add_transition_state.push((e, transition_state.clone(), TransitionStateContext{ target: attacking.0 }));
                        break;
                    }
                }
            }
        }
    }

    for (e, friction) in to_add_friction {
        handle.world.insert_one(e, friction).unwrap();
    }
    for (e, hitbox) in to_add_hitbox {
        handle.world.insert_one(e, hitbox).unwrap();
    }
    for (e, transition_state, context) in to_add_transition_state {
        handle.world.remove_one::<AttackingDash>(e).unwrap();
        let _ = handle.world.remove::<(Friction, HitBox)>(e).is_ok();
        add_transition_state(handle, e, transition_state, context);
    }
}

fn system_enemy_attacking_projectile(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState, TransitionStateContext)> = Vec::new();
    let mut to_spawn_projectile: Vec<ProjectileSpawn> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, attacking) in handle.world.query::<&AttackingProjectile>().iter() {
        let position = handle.world.get::<&Position2D>(attacking.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((attacking, enemy, position, resource)) = handle.world.query_one_mut::<(
            &mut AttackingProjectile, &Enemy, &Position2D, &mut Resource
        )>(e) {
            if !attacking.1.step(handle.delta_time) {
                continue;
            }
            let enemy_data = state.enemy_registry.get(&enemy.0).unwrap();
            let mut actionable = false;
            if resource.current > 0 {
                let direction = (target_position - position.0).normalize();
                let spawn = ProjectileSpawn::new(
                    e, "item_acorn".to_string(), enemy_data.attack_speed, direction, position.0, Vec2::new(0.4, 0.4), enemy_data.damage
                );
                to_spawn_projectile.push(spawn);
                attacking.1.reset();
                resource.current -= 1;
            } else {
                actionable = true;
            }
            if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::AttackingProjectile).clone() {
                for (transition_state, conditions) in transition_graph {
                    let mut success = true;
                    for condition in conditions {
                        match condition {
                            TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                            TransitionCondition::Actionable => success &= actionable,
                            _ => {}
                        }
                    }
                    if success {
                        to_add_transition_state.push((e, transition_state.clone(), TransitionStateContext{ target: attacking.0 }));
                        break;
                    }
                }
            }
        }
    }

    for spawn in to_spawn_projectile {
        spawn_projectile(handle, spawn);
    }
    for (e, transition_state, context) in to_add_transition_state {
        handle.world.remove_one::<AttackingProjectile>(e).unwrap();
        add_transition_state(handle, e, transition_state, context);
    }
}

fn system_enemy_escaping(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transition_state: Vec<(Entity, TransitionState, TransitionStateContext)> = Vec::new();
    let mut to_add_path: Vec<(Entity, Path)> = Vec::new();

    let mut targets: Vec<(Entity, Vec2)> = Vec::new();
    for (e, escaping) in handle.world.query::<&Escaping>().iter() {
        let position = handle.world.get::<&Position2D>(escaping.0).unwrap().0;
        targets.push((e, position));
    }
    for (e, target_position) in targets {
        if let Ok((escaping, enemy, position, collider)) = handle.world.query_one_mut::<(
            &mut Escaping, &Enemy, &Position2D, &Collider
        )>(e) {
            let enemy_data = state.enemy_registry.get(&enemy.0).unwrap().clone();
            let mut actionable = false;
            if check_line_of_sight(state, position.0, target_position, collider, enemy_data.sight_range) {
                escaping.1.reset();
            } else {
                if escaping.1.step(handle.delta_time) {
                    actionable = true;
                }
            }
            if let Some(transition_graph) = enemy_data.transition_graph.get(&TransitionState::Escaping).clone() {
                for (transition_state, conditions) in transition_graph {
                    let mut success = true;
                    for condition in conditions {
                        match condition {
                            TransitionCondition::Random(chance) => success &= state.rng.random::<f32>() <= *chance,
                            TransitionCondition::Actionable => success &= actionable,
                            _ => {}
                        }
                    }
                    if success {
                        to_add_transition_state.push((e, transition_state.clone(), TransitionStateContext{ target: escaping.0 }));
                        break;
                    }
                }
            }
            
            let direction = (position.0 - target_position).normalize();
            let escape_position = enemy_data.sight_range * direction + position.0;
            if let Some(path) = generate_path(state, position.0, escape_position, collider) {
                to_add_path.push((e, path));
            }
        }
    }

    for (e, path) in to_add_path {
        handle.world.insert_one(e, path).unwrap();
    }
    for (e, transition_state, context) in to_add_transition_state {
        handle.world.remove_one::<Escaping>(e).unwrap();
        remove_path(handle, e);
        add_transition_state(handle, e, transition_state, context);
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
                let object_data = state.object_registry.get(&object._type).unwrap();
                if object_data.collidable {
                    object_colliders.push(object.real_collider(&object_data.collider).clone());
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