use glam::*;

pub struct Camera2d {
    pub position: Vec3,
}

impl Camera2d {
    pub fn new(position: impl Into<Vec3>) -> Self {
        Self {
            position: position.into(),
        }
    }

    pub fn build_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.position);

        translation
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
        self.matrix = camera.build_matrix().to_cols_array_2d();
    }
}