use glam::*;

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
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera2d {
    pub fn default(width: f32, height: f32) -> Self {
        Self {
            updated: false,
            eye: Vec3::new(0.0, 0.0, 2.0),
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
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
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