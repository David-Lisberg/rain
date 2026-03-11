use std::ffi::CString;

use crate::engine::color::Color;

// pub fn draw_text_bounded(d: &mut RaylibDrawHandle, text: &String, font_size: i32, color: Color, width: i32, x: i32, mut y: i32) {
//     let mut current = String::new();
//     for word in text.split_whitespace() {
//         let mut temp = current.clone();
//         temp.push_str(word);
//         if measure_text(&temp, font_size) > width {
//             d.draw_text(&current, x, y, font_size, color);
//             y += (font_size as f32 * 1.3) as i32;
//             current.clear();
//         }
//         current.push_str(word);
//         current.push(' ');
//     }
//     d.draw_text(&current, x, y, font_size, color);
// }

pub fn measure_text_height(text: &String, font_size: i32, width: i32) -> i32 {
    let mut current = String::new();
    let mut height = 0;
    for word in text.split_whitespace() {
        let mut temp = current.clone();
        temp.push_str(word);
        if measure_text(&temp, font_size) > width {
            height += (font_size as f32 * 1.3) as i32;
            current.clear();
        }
        current.push_str(word);
        current.push(' ');
    }
    height + (font_size as f32 * 1.3) as i32
}

pub fn measure_text(text: &str, font_size: i32) -> i32 {
    let c_text = CString::new(text).unwrap();
    // unsafe {
    //     MeasureText(c_text.as_ptr(), font_size)
    // }
    888888
}

// pub fn draw_text_right_allignment(d: &mut RaylibDrawHandle, text: &str, mut x: i32, y: i32, font_size: i32, color: Color) {
//     let width = measure_text(text, font_size);
//     x -= width;
//     d.draw_text(text, x, y, font_size, color);
// }

// pub fn draw_text_centered(d: &mut RaylibDrawHandle, text: &str, x: i32, y: i32, font_size: i32, color: Color) {
//     let text_width = measure_text(text, font_size);
//     let x_pos = x - text_width / 2;
//     let y_pos = y - font_size / 2;
//     d.draw_text(text, x_pos, y_pos, font_size, color);
// }