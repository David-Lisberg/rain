use glam::*;

pub struct Sprite;
pub struct Model;
pub struct Visible;
#[derive(Clone, Copy)]
pub struct UVRect {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}
#[derive(Clone)]
pub struct Position2D(pub Vec2);
pub struct Position3D(pub Vec3);
pub struct Velocity2D(pub Vec2);
pub struct Acceleration2D(pub Vec2);
pub struct DepthZ(pub f32);
pub struct Scale2D(pub Vec2);
pub struct Scale3D(pub Vec3);
pub struct Rotation(pub Quat);
pub struct RotationZ(pub f32);
pub struct Flip(pub bool, pub bool);
#[derive(PartialEq)]
pub struct Direction(pub Vec2);
pub struct Dash;
pub struct Walk;
pub struct Friction(pub f32);
pub struct Priority(pub i32);

impl UVRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { offset: [x, y], scale: [width, height] }
    }

    pub fn default() -> Self {
        Self { offset: [0.0, 0.0], scale: [1.0, 1.0] }
    }
}