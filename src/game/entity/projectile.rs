use glam::Vec2;
use hecs::Entity;
use rain::engine::component::*;
use rain::engine::core::RainHandle;

use crate::DEPTH_PROJECTILE;
use crate::game::core::collision::Collider;
use crate::game::entity::damage::HitBox;
use crate::game::entity::despawn::TimerDespawn;
use crate::game::utility::timer::Timer;

pub struct Projectile;
pub struct ProjectileSpawn {
    owner: Entity,
    texture: String,
    speed: f32,
    direction: Vec2,
    position: Vec2,
    size: Vec2,
    damage: f32,
}

impl ProjectileSpawn {
    pub fn new(owner: Entity, texture: String, speed: f32, direction: Vec2, position: Vec2, size: Vec2, damage: f32) -> Self {
        Self { owner, texture, speed, direction, position, size, damage }
    }
}

pub fn system_manage_projectiles(handle: &mut RainHandle) {
    let mut to_despawn: Vec<Entity> = Vec::new();
    for (e, (_, hitbox)) in handle.world.query::<(&Projectile, Option<&HitBox>)>().iter() {
        if hitbox.is_none() {
            to_despawn.push(e);
        }
    }
    
    for e in to_despawn {
        handle.world.despawn(e).unwrap();
    }
}

pub fn spawn_projectile(handle: &mut RainHandle, spawn: ProjectileSpawn) {
    let velocity = Velocity2D(spawn.speed * spawn.direction);
    let hitbox = HitBox::new(
        spawn.damage,
        Collider::from_center(spawn.position.x, spawn.position.y, spawn.size.x, spawn.size.y),
        vec![spawn.owner],
        1
    );
    let texture = handle.fetch_texture(&spawn.texture).unwrap();

    handle.world.spawn((
        Sprite, Visible, Position2D(spawn.position), velocity, Acceleration2D(Vec2::ZERO), TimerDespawn(Timer::new(5.0)),
        texture, Scale2D(spawn.size), DepthZ(DEPTH_PROJECTILE), Priority(1), Projectile,
        hitbox,
    ));
}