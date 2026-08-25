use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use glam::Vec2;
use hecs::Entity;
use rain::engine::animation::UVRect;
use rain::engine::color::Color;
use rain::engine::component::{Position2D, Priority, Visible};
use rain::engine::core::RainHandle;
use rain::engine::mesh::ModelMesh;
use rain::engine::resource::ARRAY_512X512_ID;
use rain::engine::texture::Texture;
use rain::engine::vertex::{ModelVertex, QUAD_INDICES};
use serde::Deserialize;
use wgpu::util::DeviceExt;

use crate::game::entity::loot::LootTable;
use crate::game::player::inventory::Inventory;
use crate::{DEPTH_DIFFERENCE, DEPTH_PLAYER, State};
use crate::game::core::collision::Collider;
use crate::game::core::physics::ADJACENT_I32;
use crate::game::player::item::{Item, ToolType};
use crate::game::player::movement::Player;
use crate::game::world::chunk::{ChunkPosition, position_to_chunk_position};

pub const OBJECT_GENERATION_DISTANCE: i32 = 5;

pub struct ObjectRegistry {
    data: Vec<ObjectData>,
    ids: HashMap<String, u32>
}

impl ObjectRegistry {
    pub fn new(data: Vec<ObjectData>) -> Self {
        let mut ids: HashMap<String, u32> = HashMap::new();
        for (i, tile_data) in data.iter().enumerate() {
            ids.insert(tile_data.name.clone(), i as u32);
        }

        Self {
            data,
            ids,
        }
    }

    pub fn from_name(&self, name: &str) -> Option<&ObjectData> {
        let Some(id) = self.ids.get(name) else {
            return None;
        };
        self.data.get(*id as usize)
    }

    pub fn from_id(&self, id: u32) -> Option<&ObjectData> {
        self.data.get(id as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.ids.get(name).cloned()
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ObjectBehavior {
    Inventory(String),
}

#[derive(Deserialize, Clone)]
pub struct ObjectDataRaw {
    pub name: String,
    pub texture: String,
    pub size: Vec2,
    pub collidable: bool,
    pub uv_rect: Option<UVRect>,
    pub loot_table: Option<LootTable>,
    pub drops: Option<Vec<(Item, i32)>>,
    pub hit_ticks: Option<i32>,
    pub break_level: Option<i32>,
    pub required_tool: Option<ToolType>,
    pub depth_layer: Option<i32>,
    pub collider: Option<Collider>,
    pub offset: Option<Vec2>,
    pub behaviors: Option<Vec<ObjectBehavior>>,
    pub placeable_on_water: Option<bool>,
    pub coverable: Option<Collider>,
}

#[derive(Clone)]
pub struct ObjectData {
    pub name: String,
    pub texture: Arc<Texture>,
    pub size: Vec2,
    pub collidable: bool,
    pub uv_rect: UVRect,
    pub loot_table: LootTable,
    pub drops: Vec<(Item, i32)>,
    pub hit_ticks: i32,
    pub break_level: i32,
    pub required_tool: ToolType,
    pub depth_z: f32,
    pub collider: Collider,
    pub offset: Vec2,
    pub behaviors: Vec<ObjectBehavior>,
    pub placeable_on_water: bool,
    pub coverable: Option<Collider>,
}

impl ObjectData {
    pub fn from_raw(handle: &mut RainHandle, raw: ObjectDataRaw) -> Self {
        let offset = raw.offset.unwrap_or(Vec2::ZERO);
        let mut collider = raw.collider.unwrap_or(Collider::new(0.0, 0.0, raw.size.x, raw.size.y));
        collider.x += offset.x;
        collider.y += offset.y;
        Self {
            name: raw.name,
            texture: handle.fetch_texture(&raw.texture).unwrap(),
            size: raw.size,
            collidable: raw.collidable,
            uv_rect: raw.uv_rect.unwrap_or(UVRect::default()),
            loot_table: raw.loot_table.unwrap_or(LootTable(Vec::new())),
            drops: raw.drops.unwrap_or(Vec::new()),
            hit_ticks: raw.hit_ticks.unwrap_or(1),
            break_level: raw.break_level.unwrap_or(1),
            required_tool: raw.required_tool.unwrap_or(ToolType::None),
            depth_z: DEPTH_PLAYER + raw.depth_layer.unwrap_or(0) as f32 * DEPTH_DIFFERENCE,
            collider,
            offset,
            behaviors: raw.behaviors.unwrap_or(Vec::new()),
            placeable_on_water: raw.placeable_on_water.unwrap_or(false),
            coverable: raw.coverable,
        }
    }

    pub fn center(&self, position: Vec2) -> Vec2 {
        position + self.size / 2.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    pub type_id: u32,
    pub position: Vec2,
    pub hit_ticks: i32,
    pub transparent: bool,
    pub uv_rect: UVRect,
    pub entity: Option<Entity>,
}

impl Object {
    pub fn from_data(handle: &mut RainHandle, state: &mut State, object_type: u32, mut position: Vec2) -> Self {
        let data = state.object_registry.from_id(object_type).unwrap();
        position += data.offset;
        let entity: Option<Entity> = if data.behaviors.is_empty() {
            None
        } else {
            let e = handle.world.spawn(());
            for behavior in &data.behaviors {
                match behavior {
                    ObjectBehavior::Inventory(ui) => {
                        let inventory_data = state.inventory_registry.get(ui).unwrap();
                        handle.world.insert_one(e, Inventory::new((inventory_data.ui.rows * inventory_data.ui.columns) as usize)).unwrap();
                    }
                }
            }
            Some(e)
        };
        Self {
            type_id: object_type,
            position,
            hit_ticks: data.hit_ticks,
            transparent: false,
            uv_rect: data.uv_rect,
            entity,
        }
    }

    // pub fn real_collider(&self, collider: &Collider) -> Collider {
    //     Collider::new(self.position.x + collider.x, self.position.y + collider.y, collider.width, collider.height)
    // }
}

// #[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
// #[serde(rename_all = "snake_case")]
// pub enum ObjectType {
//     Tree1,
//     Tree2,
//     Tree3,
//     Twig,
//     Grass,
//     Stone,
//     Flint,
//     Barrel,
// }

pub struct ObjectMesh;

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
        let object_data = state.object_registry.from_id(object.type_id).unwrap();
        let color = match object.transparent {
            true => Color::rain_color_to_array(&Color::from_f32(1.0, 1.0, 1.0, 0.5)),
            false => Color::rain_color_to_array(&Color::WHITE),
        };

        let uv_scale = [object_data.texture.uv[0] / 1.0, object_data.texture.uv[1] / 1.0];
        let uv_rect = UVRect::new(object.uv_rect.offset[0] * uv_scale[0], object.uv_rect.offset[1] * uv_scale[1], 
            object.uv_rect.scale[0] * uv_scale[0], object.uv_rect.scale[1] * uv_scale[1]);

        let vertices = vec![
            ModelVertex { position: [object.position.x, object.position.y, object_data.depth_z], 
                uv: [uv_rect.offset[0], uv_rect.scale[1]], color, layer: object_data.texture.index },
            ModelVertex { position: [object.position.x + object_data.size.x, object.position.y, object_data.depth_z], 
                uv: [uv_rect.scale[0], uv_rect.scale[1]], color, layer: object_data.texture.index },
            ModelVertex { position: [object.position.x + object_data.size.x, object.position.y + object_data.size.y, object_data.depth_z], 
                uv: [uv_rect.scale[0], uv_rect.offset[1]], color, layer: object_data.texture.index },
            ModelVertex { position: [object.position.x, object.position.y + object_data.size.y, object_data.depth_z], 
                uv: [uv_rect.offset[0], uv_rect.offset[1]], color, layer: object_data.texture.index },
        ];
        let index = if object_data.depth_z < DEPTH_PLAYER {
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
                    let object_data = state.object_registry.from_id(object.type_id).unwrap();
                    if object.transparent {
                        if !under_object(collider, object_data, object) {
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
                    let object_data = state.object_registry.from_id(object.type_id).unwrap();
                    if under_object(collider, object_data, object) {
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

fn under_object(collider: &Collider, object_data: &ObjectData, object: &Object) -> bool {
    let other_collider = object_data.collider.add_vec2(object.position);

    let Some(object_collider) = object_data.coverable.clone() else {
        return false;
    };
    let object_collider = object_collider.add_vec2(object.position);

    collider.aabb_collision(&object_collider)
}

pub fn world_position_to_object_position(world_position: Vec2) -> Vec2 {
    Vec2::new(world_position.x.floor(), world_position.y.floor())
}