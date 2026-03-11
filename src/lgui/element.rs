use std::sync::Arc;
use std::usize;
use std::{cell::RefCell, rc::Rc};
use glam::*;

use crate::engine::color::Color;
use crate::engine::texture::Texture;
use crate::engine::utility::rectangle::Rect;

use super::call::{DrawCall, DrawType, DrawOption};

use super::text::*;

#[derive(Clone, Debug)]
pub enum Allignment {
    Left,
    Right,
    BoundedLeft(i32),
    Centered,
}

#[derive(Clone, Debug)]
pub enum Scale {
    XY,
    X,
    Y,
    SquareX,
    SquareY,
    LessW,
    LessH,
    None,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Rectangle(f32, f32),
    Circle(f32),
    RectangleOutline(f32, f32, f32), /* width height thickness */
    Line(f32, f32, f32), /* end thickness */
    Text(String, i32, Allignment) /* text font size */
}

#[derive(Debug)]
pub struct Collider {
    pub pos: Vec2,
    pub scaled_pos: Vec2,
    pub shape: Shape,
    pub scaled_shape: Shape,
}

impl Collider {
    pub fn new(x: f32, y: f32, shape: Shape) -> Self {
        Collider {
            pos: Vec2::new(x, y),
            scaled_pos: Vec2::new(x, y),
            shape: shape.clone(),
            scaled_shape: shape,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Widget {
    Button,
    Label,
    Panel,
}

#[derive(Debug, Clone)]
pub enum AnchorOption {
    Width,
    Height,
    X,
    Y,
    ThickLeftX,
    ThickRightX,
    ThickTopY,
    ThickBottomY,
}

#[derive(Debug)]
pub struct Element {
    pub id: usize,
    pub _draw: bool,
    pub pos: Vec2,
    pub scaled_pos: Vec2,
    pub shape: Shape,
    pub scaled_shape: Shape,
    pub widget: Widget,
    pub components: Vec<Rc<RefCell<Element>>>,
    pub texture: Option<Arc<Texture>>,
    pub color: Option<Color>,
    pub _scale: Scale,
    pub recursive: bool,
    pub collided: bool,
    pub check_collision: bool,
    pub collider: Option<Collider>,
    pub scissor: bool,
    pub anchor: Option<(usize, AnchorOption)>
}

impl Element {
    pub fn new(
        draw: bool, pos: Vec2, shape: Shape, _type: Widget, texture: Option<Arc<Texture>>, color: Option<Color>, scale: Scale, recursive: bool, 
        check_collision: bool, collider: Option<Collider>, scissor: bool, anchor: Option<(usize, AnchorOption)>
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Element {
            id: usize::MAX,
            _draw: draw,
            pos: pos,
            scaled_pos: pos,
            shape: shape.clone(),
            scaled_shape: shape,
            widget: _type,
            components: Vec::new(),
            texture: texture,
            color: color,
            _scale: scale,
            recursive: recursive,
            collided: false,
            check_collision: check_collision,
            collider: collider,
            scissor: scissor,
            anchor: anchor,
        }))
    }

    pub fn get_draw_call(&self) -> Option<DrawCall> {
        if !self._draw {
            return None;
        }
        let draw_type = match &self.scaled_shape {
            Shape::Rectangle(dim_x, dim_y) => {
                DrawType::DrawRect(self.scaled_pos.x, self.scaled_pos.y, *dim_x, *dim_y)
            }
            Shape::Circle(radius) => {
                DrawType::DrawCircle(self.scaled_pos.x, self.scaled_pos.y, *radius)
            }
            Shape::RectangleOutline(dim_x, dim_y, thickness) => {
                DrawType::DrawRectOutline(self.scaled_pos.x, self.scaled_pos.y, *dim_x, *dim_y, *thickness)
            }
            Shape::Line(end_x, end_y, thickness) => {
                DrawType::DrawLine(self.scaled_pos.x, self.scaled_pos.y, *end_x, *end_y, *thickness)
            }
            Shape::Text(text, font_size, allign) => {
                DrawType::DrawText(self.scaled_pos.x, self.scaled_pos.y, text.clone(), *font_size, allign.clone())
            }
        };
        let texture = match &self.texture {
            Some(t) => Some(Arc::clone(t)),
            None => None,
        };
        return Some(DrawCall::new(
            draw_type,
            DrawOption::None,
            self.color,
            texture
        ))
    }
}

pub fn check_collision(position: &Vec2, shape: &Shape, point: Vec2) -> bool {
    match shape {
        Shape::Rectangle(dim_x, dim_y) => {
            let rect = Rect::new(position.x, position.y, *dim_x, *dim_y);
            return rect.check_collision_point_rec(point);
        }
        Shape::Circle(radius) => {
            return check_collision_point_circle(point, *position, *radius);
        }
        Shape::RectangleOutline(dim_x, dim_y, thickness) => {
            let outer_rect = Rect::new(position.x, position.y, *dim_x, *dim_y);
            let inner_rect = Rect::new(position.x + *thickness, position.y + *thickness,
                *dim_x - *thickness * 2.0, *dim_y - *thickness * 2.0);
            return outer_rect.check_collision_point_rec(point) && !inner_rect.check_collision_point_rec(point);
        }
        Shape::Line(_, _, _) => {}
        Shape::Text(text, font_size, allign) => {
            let width = measure_text(text, *font_size);
            let rect = match allign {
                Allignment::Left => {
                    Rect::new(position.x, position.y, width as f32, *font_size as f32)
                }
                Allignment::Right => {
                    Rect::new(position.x - width as f32, position.y, width as f32, *font_size as f32)
                }
                Allignment::BoundedLeft(width) => {
                    let height = measure_text_height(text, *font_size, *width);
                    Rect::new(position.x, position.y, *width as f32, height as f32)
                }
                Allignment::Centered => {
                    let x = position.x - (width as f32 / 2.0);
                    let y = position.y - (*font_size as f32 / 2.0);
                    Rect::new(x, y, width as f32, *font_size as f32)
                }
            };
            
            return rect.check_collision_point_rec(point);
        }
    }
    false
}

pub struct ElementBuilder {
    id: usize,
    _draw: bool,
    pos: Vec2,
    shape: Shape,
    widget: Widget,
    texture: Option<Arc<Texture>>,
    color: Option<Color>,
    _scale: Scale,
    recursive: bool,
    check_collision: bool,
    collider: Option<Collider>,
    scissor: bool,
    anchor: Option<(usize, AnchorOption)>
}

impl ElementBuilder {
    pub fn new(x: f32, y: f32, shape: Shape) -> Self {
        ElementBuilder {
            id: usize::MAX,
            _draw: true,
            pos: Vec2::new(x, y),
            shape: shape.clone(),
            widget: Widget::Label,
            texture: None,
            color: None,
            _scale: Scale::None,
            recursive: false,
            check_collision: false,
            collider: None,
            scissor: false,
            anchor: None,
        }
    }

    pub fn empty() -> Self {
        ElementBuilder {
            id: usize::MAX,
            _draw: false,
            pos: Vec2::ZERO,
            shape: Shape::Rectangle(0.0, 0.0),
            widget: Widget::Panel,
            texture: None,
            color: None,
            _scale: Scale::None,
            recursive: false,
            check_collision: false,
            collider: None,
            scissor: false,
            anchor: None,
        }
    }

    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }

    pub fn draw(mut self, draw: bool) -> Self {
        self._draw = draw;
        self
    }

    pub fn widget_type(mut self, widget_type: Widget) -> Self {
        self.widget = widget_type;
        self
    }

    pub fn texture(mut self, texture: Arc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn scale(mut self, scale: Scale) -> Self {
        self._scale = scale;
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn check_collision(mut self, check_collision: bool) -> Self {
        self.check_collision = check_collision;
        self
    }

    pub fn collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }

    pub fn scissor(mut self, scissor: bool) -> Self {
        self.scissor = scissor;
        self
    }

    pub fn anchor(mut self, id: usize, anchor_option: AnchorOption) -> Self {
        self.anchor = Some((id, anchor_option));
        self
    }

    pub fn build(self) -> Rc<RefCell<Element>> {
        Element::new(
            self._draw,
            self.pos,
            self.shape.clone(),
            self.widget.clone(),
            self.texture.clone(),
            self.color,
            self._scale.clone(),
            self.recursive,
            self.check_collision,
            self.collider,
            self.scissor,
            self.anchor,
        )
    }
}

pub fn check_collision_point_circle(point: impl Into<Vec2>, center: impl Into<Vec2>, radius: f32) -> bool {
    (point.into() - center.into()).length() < radius
}