use glam::*;

use crate::engine::{core::RainHandle, utility::transform::{framebuffer_to_ndc, ndc_to_framebuffer}};

const MAX_FOV: f32 = 180.0;

pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols(
    glam::Vec4::new(1.0, 0.0, 0.0, 0.0),
    glam::Vec4::new(0.0, 1.0, 0.0, 0.0),
    glam::Vec4::new(0.0, 0.0, 0.5, 0.0),
    glam::Vec4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera2d {
    pub updated: bool,
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    pub aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera2d {
    pub fn default(width: f32, height: f32) -> Self {
        Self {
            updated: false,
            eye: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        }
    }

    pub fn set_xy(&mut self, x: f32, y: f32) {
        self.eye.x = x;
        self.eye.y = y;
        self.target.x = x;
        self.target.y = y;
        self.updated = true;
    }

    pub fn add_xy(&mut self, x: f32, y: f32) {
        self.eye.x += x;
        self.eye.y += y;
        self.target.x += x;
        self.target.y += y;
        self.updated = true;
    }

    pub fn get_xy(&self) -> Vec2 {
        self.eye.xy()
    }

    pub fn set_z(&mut self, z: f32) {
        self.eye.z = z;
        self.updated = true;
    }

    pub fn add_z(&mut self, z: f32) {
        self.eye.z += z;
        self.updated = true;
    }

    pub fn add_fov(&mut self, fov: f32) {
        self.fovy += fov;
        if self.fovy > MAX_FOV {
            self.fovy = MAX_FOV;
        } else if self.fovy < 0.0 {
            self.fovy = 0.0;
        }
        self.updated = true;
    }

    pub fn get_fov(&self) -> f32 {
        self.fovy
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fovy = fov;
        if self.fovy > MAX_FOV {
            self.fovy = MAX_FOV;
        } else if self.fovy < 0.0 {
            self.fovy = 0.0;
        }
        self.updated = true;
    }

    pub fn camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout")
        })
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera2dUniform {
    matrix: [[f32; 4]; 4],
}

impl Camera2dUniform {
    pub fn new() -> Self {
        Self {
            matrix: Mat4::IDENTITY.to_cols_array_2d()
        }
    }

    pub fn update_matrix(&mut self, camera: &Camera2d) {
        self.matrix = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

impl RainHandle {
    pub fn screen_position_to_world_position(&self, position: Vec2) -> Vec2 {
        let inverse = self.renderer.camera.build_view_projection_matrix().inverse();
        let ndc = framebuffer_to_ndc(position, self.renderer.config.width, self.renderer.config.height);

        let near = inverse * Vec4::new(ndc.x, ndc.y, -1.0, 1.0);
        let far = inverse * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);

        let near = near.truncate() / near.w;
        let far = far.truncate() / far.w;

        let t = (0.0 - near.z) / (far.z - near.z);

        let world_position = near + t * (far - near);
        Vec2::new(world_position.x, world_position.y)
    }

    pub fn world_position_to_screen_position(&self, position: Vec2) -> Vec2 {
        let matrix = self.renderer.camera.build_view_projection_matrix();
        let world_position = Vec4::new(position.x, position.y, 0.0, 1.0);
        let clip_position = matrix * world_position;
        let ndc = clip_position.xy() / clip_position.w;
        ndc_to_framebuffer(ndc, self.renderer.config.width, self.renderer.config.height)
    }
}