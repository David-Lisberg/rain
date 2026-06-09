use std::collections::HashMap;

use hecs::Entity;
use serde::Deserialize;

pub type TransitionGraph = HashMap<TransitionState, Vec<(TransitionState, Vec<TransitionCondition>)>>;

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TransitionState {
    Idle,
    Tracking,
    AttackingDash,
    AttackingProjectile,
    Escaping,
    Digging,
}

pub struct TransitionStateContext {
    pub target: Entity,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TransitionCondition {
    NotMaxResource,
    InAttackRange,
    Actionable,
    LineOfSight,
    Random(f32),
}