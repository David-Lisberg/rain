pub fn framebuffer_to_ndc(x: f32, y: f32, width: u32, height: u32) -> (f32, f32) {
    let ndc_x = x / width as f32 * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / height as f32 * 2.0);
    (ndc_x, ndc_y)
}