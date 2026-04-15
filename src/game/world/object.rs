use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::{component::{Priority, Visible}, core::RainHandle, mesh::ModelMesh, resource::{ARRAY_256X256_ID, ResourceManager}, texture::Texture, vertex::{ModelVertex, SPRITE_QUAD_INDICES}};
use wgpu::util::DeviceExt;

use crate::{DEPTH_SMALL_OBJECT, DEPTH_TREES, State, game::{core::collision::Collider, world::chunk::{ChunkPosition, position_to_chunk_position}}};

#[derive(Debug, Clone, Copy)]
pub struct Object {
    pub _type: ObjectType,
    pub position: Vec2,
    pub hit_ticks: i32,
    pub break_level: i32,
    pub depth_z: f32,
    pub size: Vec2,
    pub collider: Collider,
    pub collidable: bool,
}

#[derive(Debug, Clone, Copy)]
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
            depth_z: DEPTH_TREES,
            size: Vec2::new(1.0, 3.0), 
            collider: Collider::new(position.x + 0.2, position.y, 0.8, 1.0),
            collidable: true,
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
        depth_z: DEPTH_SMALL_OBJECT,
        size: Vec2::new(0.6, 0.6), 
        collider: Collider::new(position.x + 0.2, position.y + 0.2, 0.6, 0.6),
        collidable: false,
    }
}

pub fn construct_object_mesh(handle: &mut RainHandle, state: &mut State) -> ModelMesh {
    let mut model_vertices: Vec<ModelVertex> = Vec::new();
    let mut model_indices: Vec<u16> = Vec::new();
    let mut objects: Vec<&Object> = Vec::new();

    let mut query = handle.world.query::<(&ChunkPosition, &ModelMesh)>();
    for (_, (chunk_position, _)) in query.iter() {
        if let Some(chunk) = state.chunks.get(chunk_position) {
            for object in &chunk.objects {
                objects.push(object);
            }
        }
    }
    
    objects.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
    for (i, object) in objects.iter().enumerate() {
        let object_texture = object._type.fetch_texture(&handle.resource_manager);

        let vertices = vec![
            ModelVertex { position: [object.position.x, object.position.y, object.depth_z], 
                uv: [0.0, object_texture.uv[1]], layer: object_texture.index },
            ModelVertex { position: [object.position.x + object.size.x, object.position.y, object.depth_z], 
                uv: [object_texture.uv[0], object_texture.uv[1]], layer: object_texture.index },
            ModelVertex { position: [object.position.x + object.size.x, object.position.y + object.size.y, object.depth_z], 
                uv: [object_texture.uv[0], 0.0], layer: object_texture.index },
            ModelVertex { position: [object.position.x, object.position.y + object.size.y, object.depth_z], 
                uv: [0.0, 0.0], layer: object_texture.index },
        ];
        let indices: Vec<u16> = SPRITE_QUAD_INDICES.iter().map(|x| x + i as u16 * 4).collect();
        
        model_vertices.extend(vertices);
        model_indices.extend(indices);
    }

    ModelMesh {
        vertices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("object_vertex_buffer"),
            contents: bytemuck::cast_slice(&model_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: handle.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("object_index_buffer"),
            contents: bytemuck::cast_slice(&model_indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        num_indices: model_indices.len() as u32,
        array_id: ARRAY_256X256_ID,
    }
}

pub fn reload_object_mesh(handle: &mut RainHandle, state: &mut State) {
    let to_remove: Vec<Entity> = handle.world.query::<&ObjectMesh>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in to_remove {
        handle.world.despawn(e).unwrap();
    }
    let mesh = construct_object_mesh(handle, state);
    handle.world.spawn((mesh, ObjectMesh, Visible, Priority(0)));
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