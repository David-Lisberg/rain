use std::sync::Arc;
use std::{collections::VecDeque, rc::Rc};

use glam::Vec2;

use crate::engine::color::Color;
use crate::engine::core::RainHandle;
use crate::engine::texture::Texture;
use crate::engine::utility::rectangle::Rect;

use super::element::Allignment;
use super::text::*;

#[derive(Debug)]
pub struct DrawCall {
    draw_type: DrawType,
    draw_option: DrawOption,
    color: Option<Color>,
    texture: Option<Arc<Texture>>,
}

impl DrawCall {
    pub fn new(draw_type: DrawType, draw_option: DrawOption, color: Option<Color>, texture: Option<Arc<Texture>>) -> Self {
        Self {
            draw_type: draw_type,
            draw_option: draw_option,
            color: color,
            texture: texture,
        }
    }

    pub fn option(draw_option: DrawOption) -> Self {
        Self::new(DrawType::None, draw_option, None, None)
    }
}

#[derive(Debug, Clone)]
pub enum DrawType {
    None,
    DrawRect(f32, f32, f32, f32), /* x, y, width, height */
    DrawRectOutline(f32, f32, f32, f32, f32), /* x, y, width, height, thickness */
    DrawCircle(f32, f32, f32), /* x, y, radius */
    DrawLine(f32, f32, f32, f32, f32), /* start x, start y, end x, end y, thickness */
    DrawText(f32, f32, String, i32, Allignment), /* x, y, text, font size, allignment */
}

#[derive(Debug)]
pub enum DrawOption {
    None,
    BeginScissorMode(i32, i32, i32, i32), /* x, y, width, height */
    EndScissorMode,
}

// pub fn raylib_draw(d: &mut RaylibDrawHandle, draw_calls: &mut VecDeque<DrawCall>) {
//     let mut scissor: Option<(i32, i32, i32, i32)> = None;
//     while let Some(draw_call) = draw_calls.pop_front() {
//         match draw_call.draw_option {
//             DrawOption::BeginScissorMode(x, y, width, height) => {
//                 scissor = Some((x, y, width, height));
//             }
//             DrawOption::EndScissorMode => {
//                 scissor = None;
//             }
//             DrawOption::None => {
//                 match scissor {
//                     Some((x, y, width, height)) => {
//                         raylib::drawing::RaylibScissorModeExt::draw_scissor_mode(d, x, y, width, height, 
//                             |mut s| {
//                                 handle_raylib_draw_call(&mut s, &draw_call);
//                             }
//                         );
//                     }
//                     None => handle_raylib_draw_call(d, &draw_call),
//                 }
//             }
//         }
        
//     }
// }

pub fn rain_draw(handle: &mut RainHandle, draw_calls: &mut VecDeque<DrawCall>) {
    let mut scissor: Option<(i32, i32, i32, i32)> = None;
    while let Some(draw_call) = draw_calls.pop_front() {
        match draw_call.draw_option {
            DrawOption::BeginScissorMode(x, y, width, height) => {
                scissor = Some((x, y, width, height));
            }
            DrawOption::EndScissorMode => {
                scissor = None;
            }
            DrawOption::None => {
                match scissor {
                    Some((x, y, width, height)) => {
                        // raylib::drawing::RaylibScissorModeExt::draw_scissor_mode(d, x, y, width, height, 
                        //     |mut s| {
                        //         handle_raylib_draw_call(&mut s, &draw_call);
                        //     }
                        // );
                    }
                    None => handle_rain_draw_call(handle, &draw_call),
                }
            }
        }
        
    }
}

// fn handle_raylib_draw_call(d: &mut RaylibDrawHandle, draw_call: &DrawCall) {
//     let origin = Vec2::ZERO;
//     let color = match draw_call.color {
//         Some(c) => c,
//         None => Color::WHITE,
//     };
//     match draw_call.draw_type.clone() {
//         DrawType::DrawCircle(x, y, radius) => {
//             d.draw_circle_v(Vec2::new(x, y), radius, color);
//         }
//         DrawType::DrawLine(start_x, start_y, end_x, end_y, thickness) => {
//             d.draw_line_ex(Vec2::new(start_x, start_y), Vec2::new(end_x, end_y), thickness, color);
//         }
//         DrawType::DrawRect(x, y, width, height) => {
//             let rect = Rect::new(x, y, width, height);
//             match &draw_call.texture {
//                 Some(t) => {
//                     let src = Rect::new(0.0, 0.0, t.width as f32, t.height as f32);
//                     d.draw_texture_pro(t.as_ref(), src, rect, origin, 0.0, color);
//                 }
//                 None => d.draw_rectangle_pro(rect, origin, 0.0, color),
//             }
//         }
//         DrawType::DrawRectOutline(x, y, width, height, thickness) => {
//             let rect = Rectangle::new(x, y, width, height);
//             d.draw_rectangle_lines_ex(rect, thickness, color);
//         }
//         DrawType::DrawText(x, y, text, font_size, allignment) => {
//             match allignment {
//                 Allignment::Left => {
//                     d.draw_text(&text, x as i32, y as i32, font_size, color);
//                 }
//                 Allignment::Right => {
//                     draw_text_right_allignment(d, &text, x as i32, y as i32, font_size, color);
//                 }
//                 Allignment::BoundedLeft(width) => {
//                     draw_text_bounded(d, &text, font_size, color, width, x as i32, y as i32);
//                 }
//                 Allignment::Centered => {
//                     draw_text_centered(d, &text, x as i32, y as i32, font_size, color);
//                 }
//             }
//         }
//         DrawType::None => {}
//     }
// }

fn handle_rain_draw_call(handle: &mut RainHandle, draw_call: &DrawCall) {
    let origin = Vec2::ZERO;
    let color = match draw_call.color {
        Some(c) => c,
        None => Color::WHITE,
    };
    match draw_call.draw_type.clone() {
        DrawType::DrawCircle(x, y, radius) => {
            // d.draw_circle_v(Vec2::new(x, y), radius, color);
        }
        DrawType::DrawLine(start_x, start_y, end_x, end_y, thickness) => {
            // d.draw_line_ex(Vec2::new(start_x, start_y), Vec2::new(end_x, end_y), thickness, color);
        }
        DrawType::DrawRect(x, y, width, height) => {
            let rect = Rect::new(x, y, width, height);
            match &draw_call.texture {
                Some(t) => {
                    handle.draw_texture(rect, Arc::clone(t), color);
                }
                None => handle.draw_rectangle(rect, color),
            }
        }
        DrawType::DrawRectOutline(x, y, width, height, thickness) => {
            // let rect = Rectangle::new(x, y, width, height);
            // d.draw_rectangle_lines_ex(rect, thickness, color);
        }
        DrawType::DrawText(x, y, text, font_size, allignment) => {
            match allignment {
                Allignment::Left => {
                    // d.draw_text(&text, x as i32, y as i32, font_size, color);
                }
                Allignment::Right => {
                    // draw_text_right_allignment(d, &text, x as i32, y as i32, font_size, color);
                }
                Allignment::BoundedLeft(width) => {
                    // draw_text_bounded(d, &text, font_size, color, width, x as i32, y as i32);
                }
                Allignment::Centered => {
                    // draw_text_centered(d, &text, x as i32, y as i32, font_size, color);
                }
            }
        }
        DrawType::None => {}
    }
}