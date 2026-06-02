use std::collections::HashSet;
use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::color::Color;
use rain::engine::component::{Position2D, Priority, Visible};
use rain::engine::core::RainHandle;
use rain::engine::mesh::ModelMesh;
use rain::engine::resource::{ARRAY_512X512_ID, ResourceManager};
use rain::engine::texture::Texture;
use rain::engine::vertex::{ModelVertex, QUAD_INDICES};
use serde::Deserialize;
use wgpu::util::DeviceExt;

use crate::{State, DEPTH_PLAYER, DEPTH_SMALL_OBJECT, DEPTH_TREES};
use crate::game::core::collision::Collider;
use crate::game::core::physics::ADJACENT_I32;
use crate::game::player::item::ToolType;
use crate::game::player::movement::Player;
use crate::game::world::chunk::{ChunkPosition, position_to_chunk_position};

pub const OBJECT_GENERATION_DISTANCE: i32 = 5;

#[derive(Debug, Clone, Copy)]
pub struct Object {
    pub _type: ObjectType,
    pub position: Vec2,
    pub hit_ticks: i32,
    pub break_level: i32,
    pub required_tool: ToolType,
    pub depth_z: f32,
    pub size: Vec2,
    pub collider: Collider,
    pub collidable: bool,
    pub transparent: bool,
}

impl Object {
    pub fn center(&self) -> Vec2 {
        self.position + self.size / 2.0
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ObjectType {
    Tree1,
    Twig,
    Grass,
    Stone,
    Flint,
}

pub struct ObjectMesh;

impl ObjectType {
    pub fn fetch_texture(&self, resource_manager: &ResourceManager) -> Arc<Texture> {
        match self {
            ObjectType::Tree1 => resource_manager.fetch_texture("object_tree1").unwrap(),
            ObjectType::Twig => resource_manager.fetch_texture("object_twig").unwrap(),
            ObjectType::Grass => resource_manager.fetch_texture("object_grass").unwrap(),
            ObjectType::Stone => resource_manager.fetch_texture("object_stone").unwrap(),
            ObjectType::Flint => resource_manager.fetch_texture("object_flint").unwrap(),
        }
    }
}

pub fn construct_object_default(_type: ObjectType, position: Vec2) -> Object {
    match _type {
        ObjectType::Tree1 => Object { 
            _type, 
            position, 
            hit_ticks: 3,
            break_level: 1,
            required_tool: ToolType::Axe,
            depth_z: DEPTH_TREES,
            size: Vec2::new(1.0, 3.0), 
            collider: Collider::new(position.x + 0.2, position.y, 0.8, 1.0),
            collidable: true,
            transparent: false,
        },
        ObjectType::Twig => object_small_default(_type, position),
        ObjectType::Grass => object_small_default(_type, position),
        ObjectType::Stone => object_small_default(_type, position),
        ObjectType::Flint => object_small_default(_type, position),
    }
}

fn object_small_default(_type: ObjectType, position: Vec2) -> Object {
    Object { 
        _type, 
        position: Vec2::new(position.x + 0.2, position.y + 0.2), 
        hit_ticks: 1,
        break_level: 0,
        required_tool: ToolType::None,
        depth_z: DEPTH_SMALL_OBJECT,
        size: Vec2::new(0.6, 0.6), 
        collider: Collider::new(position.x + 0.2, position.y + 0.2, 0.6, 0.6),
        collidable: false,
        transparent: false,
    }
}

pub fn construct_object_mesh(handle: &mut RainHandle, state: &mut State) -> Vec<ModelMesh> {
    let mut model_vertices: Vec<Vec<ModelVertex>> = vec![Vec::new(); 2];
    let mut model_indices: Vec<Vec<u32>> = vec![Vec::new(); 2];
    let mut meshes: Vec<ModelMesh> = Vec::new();
    let mut objects: Vec<&Object> = Vec::new();

    let mut player_position: Option<ChunkPosition> = None;
    for (_, (_, position)) in handle.world.query::<(&Player, &Position2D)>().iter() {
        let chunk_position = position_to_chunk_position(position.0.x, position.0.y);
        player_position = Some(chunk_position);
    }
    let player_position = player_position.unwrap();
    for (_, (chunk_position, _)) in handle.world.query::<(&ChunkPosition, &ModelMesh)>().iter() {
        if chunk_position.x <= player_position.x + OBJECT_GENERATION_DISTANCE / 2 && chunk_position.x >= player_position.x - OBJECT_GENERATION_DISTANCE / 2 &&
           chunk_position.y <= player_position.y + OBJECT_GENERATION_DISTANCE / 2 && chunk_position.y >= player_position.y - OBJECT_GENERATION_DISTANCE / 2 {
            if let Some(chunk) = state.chunks.get(chunk_position) {
                for object in &chunk.objects {
                    objects.push(object);
                }
            }
        }
    }
    let mut indices_start: Vec<u32> = vec![0, 0];
    
    objects.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
    for object in objects.iter() {
        let object_texture = object._type.fetch_texture(&handle.resource_manager);
        let color = match object.transparent {
            true => Color::rain_color_to_array(&Color::from_f32(1.0, 1.0, 1.0, 0.5)),
            false => Color::rain_color_to_array(&Color::WHITE),
        };

        let vertices = vec![
            ModelVertex { position: [object.position.x, object.position.y, object.depth_z], 
                uv: [0.0, object_texture.uv[1]], color, layer: object_texture.index },
            ModelVertex { position: [object.position.x + object.size.x, object.position.y, object.depth_z], 
                uv: [object_texture.uv[0], object_texture.uv[1]], color, layer: object_texture.index },
            ModelVertex { position: [object.position.x + object.size.x, object.position.y + object.size.y, object.depth_z], 
                uv: [object_texture.uv[0], 0.0], color, layer: object_texture.index },
            ModelVertex { position: [object.position.x, object.position.y + object.size.y, object.depth_z], 
                uv: [0.0, 0.0], color, layer: object_texture.index },
        ];
        let index = if object.depth_z < DEPTH_PLAYER {
            0
        } else {
            1
        };
        
        let indices: Vec<u32> = QUAD_INDICES.iter().map(|x| x + indices_start[index]).collect();
        model_vertices[index].extend(vertices);
        model_indices[index].extend(indices);
        indices_start[index] += 4;
    }

    for (vertices, indices) in model_vertices.iter().zip(model_indices.iter()) {
        meshes.push(ModelMesh {
            vertices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("object_vertex_buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            indices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("object_index_buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            num_indices: indices.len() as u32,
            array_id: ARRAY_512X512_ID,
        })
    }
    meshes
}

pub fn reload_object_mesh(handle: &mut RainHandle, state: &mut State) {
    let to_remove: Vec<Entity> = handle.world.query::<&ObjectMesh>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in to_remove {
        handle.world.despawn(e).unwrap();
    }
    let mut meshes = construct_object_mesh(handle, state);
    handle.world.spawn((meshes.remove(0), ObjectMesh, Visible, Priority(0)));
    handle.world.spawn((meshes.remove(0), ObjectMesh, Visible, Priority(2)));
}

pub fn destroy_object(state: &mut State, object: &Object, hit_ticks: i32) -> bool {
    let chunk_position = position_to_chunk_position(object.position.x, object.position.y);
    if let Some(chunk) = state.chunks.get_mut(&chunk_position) {
        let mut to_remove: Option<usize> = None;
        for (i, chunk_object) in chunk.objects.iter_mut().enumerate() {
            if chunk_object.position == object.position {
                chunk_object.hit_ticks -= hit_ticks;
                if chunk_object.hit_ticks <= 0 {
                    to_remove = Some(i);
                }
                break;
            }
        }
        if let Some(i) = to_remove {
            chunk.objects.remove(i);
            return true;
        }
    }
    false
}

pub fn system_object_transparency(handle: &mut RainHandle, state: &mut State) {
    let mut to_add_transparent: Vec<ChunkPosition> = Vec::new();
    let mut updated: bool = false;

    for (_, (_, position, collider)) in handle.world.query::<(&Player, &Position2D, &Collider)>().iter() {
        for chunk_position in std::mem::take(&mut state.transparent_object_chunks) {
            if let Some(chunk) = state.chunks.get_mut(&chunk_position) {
                for object in &mut chunk.objects {
                    if object.transparent {
                        if !under_object(collider, object) {
                            object.transparent = false;
                            updated = true;
                        } else {
                            to_add_transparent.push(chunk_position);
                        }
                    }
                }
            }
        }
        let chunk_position = position_to_chunk_position(position.0.x, position.0.y);
        for adjacent in ADJACENT_I32 {
            let adjacent_position = ChunkPosition::new(chunk_position.x + adjacent.0, chunk_position.y + adjacent.1);
            if let Some(chunk) = state.chunks.get_mut(&adjacent_position) {
                for object in &mut chunk.objects {
                    if under_object(collider, object) {
                        if !object.transparent {
                            updated = true;
                        }
                        object.transparent = true;
                        to_add_transparent.push(chunk_position);
                    }
                }
            }
        }
    }

    let mut remove_duplicates = HashSet::new();
    to_add_transparent.retain(|x| remove_duplicates.insert(x.clone()));

    for position in to_add_transparent {
        state.transparent_object_chunks.push(position);
    }
    if updated {
        reload_object_mesh(handle, state);
    }
}

fn under_object(collider: &Collider, object: &Object) -> bool {
    let object_collider = match object._type {
        ObjectType::Tree1 => Collider::new(
            object.collider.x + 0.2, object.collider.y + 0.2, object.collider.width - 0.4, object.size.y - object.collider.y + object.position.y - 0.6
        ),
        _ => return false,
    };
    collider.aabb_collision(&object_collider)
}