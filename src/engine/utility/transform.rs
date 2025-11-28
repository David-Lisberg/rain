use glam::*;

pub fn framebuffer_to_ndc(point: impl Into<Vec2>, width: u32, height: u32) -> Vec2 {
    let point = point.into();
    let ndc_x = point.x / width as f32 * 2.0 - 1.0;
    let ndc_y = 1.0 - (point.y / height as f32 * 2.0);
    Vec2::new(ndc_x, ndc_y)
}

pub fn rotate_around_pivot(point: impl Into<Vec2>, pivot: impl Into<Vec2>, degrees: f32) -> Vec2 {
    let angle = degrees.to_radians();

    let pivot = pivot.into();

    let translate_to_origin = Mat3::from_translation(-pivot);
    let rotate = Mat3::from_rotation_z(angle);
    let translate_back = Mat3::from_translation(pivot);

    let transform = translate_back * rotate * translate_to_origin;

    transform.transform_point2(point.into()).into()
}