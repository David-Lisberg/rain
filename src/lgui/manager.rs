use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use glam::Vec2;

use crate::engine::color::Color;
use crate::engine::core::RainHandle;
use crate::engine::texture::Texture;

use super::text::measure_text;

use super::call::DrawOption;

use super::call::rain_draw;

use super::call::DrawCall;

use super::layout::*;
use super::element::*;

pub struct ElementManager {
    pub screens: Vec<LayoutManager>,
    pub screen_map: HashMap<String, usize>,
    pub current_screen: usize,
    pub current_screens: Vec<usize>,

    pub default_width: f32,
    pub default_height: f32,

    pub elements: Vec<Rc<RefCell<Element>>>,
    pub element_map: HashMap<usize, usize>,
    pub current_id: usize,

    pub pending_draw_calls: VecDeque<DrawCall>,

    pub textures: HashMap<String, Arc<Texture>>
}

impl ElementManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        ElementManager {
            screens: Vec::new(),
            screen_map: HashMap::new(),
            current_screen: usize::MAX,
            current_screens: Vec::new(),
            default_width: screen_width,
            default_height: screen_height,
            elements: Vec::new(),
            element_map: HashMap::new(),
            current_id: 0,
            pending_draw_calls: VecDeque::new(),
            textures: HashMap::new(),
        }
    }

    pub fn add_screen(&mut self, name: &str) {
        let index = self.screens.len();
        self.screens.push(LayoutManager::new(self.default_width, self.default_height, Scale::XY));
        
        self.current_screen = index;
        self.current_screens = vec![index];
        self.screen_map.insert(name.to_string(), index);
    }

    pub fn set_screen(&mut self, name: &str) {
        if let Some(index) = self.screen_map.get(name) {
            self.current_screen = *index;
            self.current_screens = vec![*index];
        }
    }

    pub fn push_screen(&mut self, name: &str) {
        if let Some(index) = self.screen_map.get(name) {
            self.current_screen = *index;
            self.current_screens.push(*index);
        }
    }

    pub fn pop_screen(&mut self) {
        if !self.current_screens.is_empty() {
            self.current_screens.pop();
            if let Some(screen) = self.current_screens.last() {
                self.current_screen = *screen;
            }
        }
    }

    pub fn index_element(&mut self, element: &Rc<RefCell<Element>>) -> usize {
        let id = loop {
            if let Some(_) = self.element_map.get(&self.current_id) {
                self.current_id += 1;
            } else {
                break self.current_id
            }
        };
        self.current_id += 1;

        let mut e = element.borrow_mut();
        e.id = id;

        let index = self.elements.len();
        self.elements.push(Rc::clone(element));
        self.element_map.insert(id, index);

        id
    }

    pub fn add_element_layout(&mut self, element: &Rc<RefCell<Element>>, screen_index: usize, layout_name: &str) -> Option<usize> {
        if let Some(manager) = self.screens.get(screen_index) {
            if let Some(layout) = manager.fetch_layout(layout_name) {
                let mut l = layout.borrow_mut();
                let id = self.index_element(element);
                l.components.push(Rc::clone(element));
                return Some(id);
            }
        }
        None
    }

    pub fn add_element(&mut self, element: &Rc<RefCell<Element>>, name: &str) -> Option<usize> {
        self.add_element_layout(element, self.current_screen, name)
    }

    pub fn add_element_child(&mut self, parent_id: usize, element: &Rc<RefCell<Element>>) -> Option<usize> {
        if let Some(parent_index) = self.element_map.get(&parent_id) {
            if let Some(parent) = self.elements.get(*parent_index) {
                {
                    let mut p = parent.borrow_mut();
                    p.components.push(Rc::clone(element));
                }
                return Some(self.index_element(element));
            }
        }
        None
    }

    pub fn add_element_fit_color(&mut self, screen_index: usize, layout_name: &str, color: Color) -> Option<usize> {
        if let Some(manager) = self.screens.get(screen_index) {
            if let Some(layout) = manager.fetch_layout(layout_name) {
                let mut l = layout.borrow_mut();
        
                let element = ElementBuilder::new(l.pos.x, l.pos.y, Shape::Rectangle(l.dim.x, l.dim.y))
                    .color(color)
                    .check_collision(true)
                    .build();
                let id = self.index_element(&element);
                l.components.push(Rc::clone(&element));
                return Some(id);
            }
        }
        None
    }

    pub fn add_element_fit_color_current(&mut self, name: &str, color: Color) -> Option<usize> {
        self.add_element_fit_color(self.current_screen, name, color)
    }

    pub fn element_immediate(&mut self, element: &Rc<RefCell<Element>>, name: &str) -> Option<usize> {
        let mut id: Option<usize> = None;
        let mut draw_calls: VecDeque<DrawCall> = VecDeque::new();
        if let Some(manager) = self.screens.get(self.current_screen) {
            if let Some(layout) = manager.fetch_layout(name) {
                let mut l = layout.borrow_mut();
                id = Some(self.index_element(element));
                l.components.push(Rc::clone(element));
                self.scale_element_recursive(&l, element);
                self.load_draw_calls_recursive(&l, element, &mut draw_calls);
            }
        }
        self.pending_draw_calls.extend(draw_calls);
        id
    }

    pub fn button_immediate(&mut self, point: Vec2, x: f32, y: f32, shape: Shape, name: &str) -> bool {
        if let Some(manager) = self.screens.get(self.current_screen) {
            if let Some(layout) = manager.fetch_layout(name) {
                let element = ElementBuilder::new(x, y, shape).build();
                let mut l = layout.borrow_mut();
                self.index_element(&element);
                l.components.push(Rc::clone(&element));
                self.scale_element_recursive(&l, &element);
                let e = element.borrow();
                return check_collision(&e.scaled_pos, &e.scaled_shape, point);
            }
        }
        false
    }

    pub fn button_immediate_layout(&mut self, point: Vec2, name: &str) -> bool {
        if let Some(manager) = self.screens.get(self.current_screen) {
            if let Some(layout) = manager.fetch_layout(name) {
                let l = layout.borrow();
                return check_collision(&l.scaled_pos, &Shape::Rectangle(l.scaled_dim.x, l.scaled_dim.y), point);
            }
        }
        false
    }

    pub fn sub_layout_immediate(&mut self, layout_name: &str, x: f32, y: f32, width: f32, height: f32, scale: Scale, scissor: bool, recursive: bool, name: &str) {
        if let Some(mananager) = self.screens.get_mut(self.current_screen) {
            if let Some(layout) = mananager.fetch_layout(layout_name) {
                let mut l = layout.borrow_mut();
                let new_layout = Rc::new(RefCell::new(Layout::sub_layout(
                    Vec2::new(x, y), Vec2::new(width, height), scale, scissor, recursive
                )));
                let index = mananager.layout.len();
                l.children.push(index);
                mananager.layout.push(Rc::clone(&new_layout));
                mananager.layout_map.insert(name.to_string(), index);
                let mut n = new_layout.borrow_mut();
                mananager.scale_sub_layout(&l, &mut n);
            }
        }
    }

    pub fn remove_element(&mut self, id: usize) {
        if let Some(element) = hash_map_link_remove(&id, &mut self.element_map, &mut self.elements) {
            let e = element.borrow();
            self.remove_element_recursive(&e);
        }
    }

    fn remove_element_recursive(&mut self, element: &Element) {
        for component in &element.components {
            let c = component.borrow();
            hash_map_link_remove(&c.id, &mut self.element_map, &mut self.elements);
            self.remove_element_recursive(&c);
        }
    }

    pub fn remove_element_layout(&mut self, id: usize, screen_index: usize, layout_name: &str) {
        if let Some(manager) = self.screens.get(screen_index) {
            if let Some(layout) = manager.fetch_layout(layout_name) {
                let l = layout.borrow();
                for component in &l.components {
                    let c = component.borrow();
                    if c.id == id {
                        self.remove_element(id);
                    }
                }
            }
        }
    }

    pub fn remove_element_layout_current(&mut self, id: usize, layout_name: &str) {
        self.remove_element_layout(id, self.current_screen, layout_name);
    }

    pub fn split_layout(&mut self, screen_index: usize, layout_name: &str, scale: (Scale, Scale), new_name: (&str, &str), split: f32, axis: Axis) {
        if let Some(manager) = self.screens.get_mut(screen_index) {
            manager.split(layout_name, scale, new_name, split, axis);
        }
    }

    pub fn split_layout_current(&mut self, name: &str, scale: (Scale, Scale), new_name: (&str, &str), split: f32, axis: Axis) {
        self.split_layout(self.current_screen, name, scale, new_name, split, axis);
    }

    pub fn add_sub_layout_index(&mut self, screen_index: usize, layout_name: &str, x: f32, y: f32, width: f32, height: f32, scale: Scale, name: &str) {
        if let Some(mananager) = self.screens.get_mut(screen_index) {
            if let Some(layout) = mananager.fetch_layout(layout_name) {
                let mut l = layout.borrow_mut();
                let new_layout = Rc::new(RefCell::new(Layout::sub_layout(
                    Vec2::new(x, y), Vec2::new(width, height), scale, false, false
                )));
                let index = mananager.layout.len();
                l.children.push(index);
                mananager.layout.push(new_layout);
                mananager.layout_map.insert(name.to_string(), index);
            }
        }
    }

    pub fn add_sub_layout(&mut self, layout_name: &str, x: f32, y: f32, width: f32, height: f32, scale: Scale, name: &str) {
        self.add_sub_layout_index(self.current_screen, layout_name, x, y, width, height, scale, name);
    }

    pub fn scale_layout(&mut self, index: usize, current_width: f32, current_height: f32) {
        if let Some(manager) = self.screens.get_mut(index) {
            manager.scale_from_root(&Vec2::new(current_width, current_height), &Vec2::new(self.default_width, self.default_height));
        }
        if let Some(manager) = self.screens.get(index) { /* handle borrow checker */
            let mut to_search: VecDeque<usize> = VecDeque::new();
            for (i, layout) in manager.layout.iter().enumerate() {
                let l = layout.borrow();
                if !l.sub_layout {
                    to_search.push_back(i);
                }
            }
            while let Some(i) = to_search.pop_front() {
                if let Some(layout) = manager.layout.get(i) {
                    let l = layout.borrow();
                    if l.sub_layout {
                        for component in &l.components {
                            self.scale_sub_layout_elements(&l, component);
                        }
                    } else {
                        for component in &l.components {
                            self.scale_element_recursive(&l, component);
                        }
                    }
                    for child_index in &l.children {
                        if let Some(child) = manager.layout.get(*child_index) {
                            let mut c = child.borrow_mut();
                            manager.scale_sub_layout(&l, &mut c);
                        }
                        to_search.push_back(*child_index);
                    }
                }
            }
        }
    }

    fn scale_sub_layout_elements(&self, layout: &Layout, element: &Rc<RefCell<Element>>) {
        let mut e = element.borrow_mut();

        let scale = Vec2::new(layout.scaled_dim.x / layout.dim.x, layout.scaled_dim.y / layout.dim.y);
        e.scaled_pos.x = (e.pos.x - layout.pos.x) * scale.x + layout.scaled_pos.x;
        e.scaled_pos.y = (e.pos.y - layout.pos.y) * scale.y + layout.scaled_pos.y;

        let shape = e.shape.clone();
        match (&mut e.scaled_shape, shape) {
            (Shape::Rectangle(dim1_x, dim1_y), Shape::Rectangle(dim2_x, dim2_y)) => {
                *dim1_x = dim2_x * scale.y;
                *dim1_y = dim2_y * scale.y;
            }
            (Shape::RectangleOutline(dim1_x, dim1_y, b1), Shape::RectangleOutline(dim2_x, dim2_y, b2)) => {
                *dim1_x = dim2_x * scale.x;
                *dim1_y = dim2_y * scale.y;
                *b1 = b2 * scale.y;
            }
            (Shape::Line(end1_x, end1_y, _), Shape::Line(end2_x, end2_y, _)) => {
                *end1_x = layout.scaled_pos.x + (end2_x - layout.pos.x) * scale.x;
                *end1_y = layout.scaled_pos.y + (end2_y - layout.pos.y) * scale.y;
            }
            (Shape::Circle(rad1), Shape::Circle(rad2)) => {
                *rad1 = rad2 * scale.y;
            }
            (Shape::Text(_, font_size1, allign1), Shape::Text(_, font_size2, allign2)) => {
                *font_size1 = (font_size2 as f32 * scale.y) as i32;
                match (allign1, allign2) {
                    (Allignment::BoundedLeft(width1), Allignment::BoundedLeft(width2)) => {
                        *width1 = (width2 as f32 * scale.x) as i32;
                    }
                    _ => {}
                }
            }
            _ => {}
        }        

        for other_e in &e.components {
            self.scale_sub_layout_elements(layout, other_e);
        }
    }

    pub fn scale_layout_current(&mut self, current_width: f32, current_height: f32) {
        self.scale_layout(self.current_screen, current_width, current_height);
    }

    fn scale_element_recursive(&self, layout: &Layout, element: &Rc<RefCell<Element>>) {
        let mut e = element.borrow_mut();

        let scale = Vec2::new(layout.scaled_dim.x / layout.dim.x, layout.scaled_dim.y / layout.dim.y);
        let shape = e.shape.clone();
        let anchor = e.anchor.clone();
        let scale_option = if layout.recursive {
            layout.scale.clone()
        } else {
            e._scale.clone()
        };

        self.scale_position(layout, scale_option.clone(), e.pos, &mut e.scaled_pos, shape, anchor.clone(), scale);
        let pos = e.pos.clone();
        let scaled_pos = e.scaled_pos.clone();

        self.scale_shape(layout, scale_option.clone(), e.shape.clone(), &mut e.scaled_shape, pos, scaled_pos, scale);
        if let Some(collider) = &mut e.collider {
            collider.scaled_pos.x = layout.scaled_pos.x + (collider.pos.x - layout.pos.x) * scale.x;
            collider.scaled_pos.y = layout.scaled_pos.y + (collider.pos.y - layout.pos.y) * scale.y;
            let shape = collider.shape.clone();
            self.scale_position(layout, scale_option.clone(), collider.pos, &mut collider.pos, shape, anchor, scale);
            self.scale_shape(layout, scale_option, collider.shape.clone(), &mut collider.scaled_shape, collider.pos, collider.scaled_pos, scale);
        }
        if e.recursive {
            for component in &e.components {
                self.scale_from_parent(&e, component);
            }
        } else {
            for component in &e.components {
                self.scale_element_recursive(layout, component);
            }
        }
    }

    fn scale_position(
        &self, layout: &Layout, scale_option: Scale, pos: Vec2, scaled_pos: &mut Vec2, shape: Shape, anchor: Option<(usize, AnchorOption)>, scale: Vec2
    ) {
        if let Some((id, anchor_option)) = anchor {
            if let Some(e) = self.fetch_element(id) {
                match anchor_option {
                    AnchorOption::Width => {
                        if let (Shape::Rectangle(w_1, _), Shape::Rectangle(w_2, _)) = (&e.shape, &e.scaled_shape) {
                            let scale_x = (pos.x - e.pos.x) / w_1;
                            scaled_pos.x = e.scaled_pos.x + w_2 * scale_x
                        } else {
                            scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                        }
                        scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                    }
                    AnchorOption::Height => {
                        if let (Shape::Rectangle(_, h_1), Shape::Rectangle(_, h_2)) = (&e.shape, &e.scaled_shape) {
                            let scale_y = (pos.y - e.pos.y) / h_1;
                            scaled_pos.y = e.scaled_pos.y + h_2 * scale_y
                        } else {
                            scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                        }
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                    }
                    AnchorOption::X => {
                        let scale_x = e.scaled_pos.x / e.pos.x;
                        scaled_pos.x = e.scaled_pos.x + (pos.x - e.pos.x) * scale_x;
                        scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                    }
                    AnchorOption::Y => {
                        let scale_y = e.scaled_pos.y / e.pos.y;
                        scaled_pos.y = e.scaled_pos.y + (pos.y - e.pos.y) * scale_y;
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                    }
                    AnchorOption::ThickLeftX => {
                        if let (Shape::RectangleOutline(_, _, t_1), Shape::RectangleOutline(_, _, t_2)) = (&e.shape, &e.scaled_shape) {
                            let scale_x = (pos.x - e.pos.x) / t_1;
                            scaled_pos.x = e.scaled_pos.x + t_2 * scale_x
                        } else {
                            scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                        }
                        scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                    }
                    AnchorOption::ThickRightX => {
                        if let (Shape::RectangleOutline(w_1, _, t_1), Shape::RectangleOutline(w_2, _, t_2)) = (&e.shape, &e.scaled_shape) {
                            let scale_x = match shape {
                                Shape::Rectangle(w_3, _) => (e.pos.x + w_1 - (pos.x + w_3)) / t_1,
                                Shape::Text(t, f, a) => match a {
                                    Allignment::BoundedLeft(w_3) => (e.pos.x + w_1 - (pos.x + w_3 as f32)) / t_1,
                                    Allignment::Centered => (e.pos.x + w_1 - (pos.x + measure_text(&t, f) as f32 / 2.0)) / t_1,
                                    Allignment::Right => (e.pos.x + w_1 - pos.x) / t_1,
                                    Allignment::Left => (e.pos.x + w_1 - (pos.x + measure_text(&t, f) as f32)) / t_1,
                                }
                                _ => 0.0,
                            };
                            scaled_pos.x = e.scaled_pos.x + w_2 - t_2 * scale_x
                        } else {
                            scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                        }
                        scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                    }
                    AnchorOption::ThickTopY => {
                        if let (Shape::RectangleOutline(_, _, t_1), Shape::RectangleOutline(_, _, t_2)) = (&e.shape, &e.scaled_shape) {
                            let scale_y = (pos.y - e.pos.y) / t_1;
                            scaled_pos.y = e.scaled_pos.y + t_2 * scale_y
                        } else {
                            scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                        }
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                    }
                    AnchorOption::ThickBottomY => {
                        if let (Shape::RectangleOutline(_, h_1, t_1), Shape::RectangleOutline(_, h_2, t_2)) = (&e.shape, &e.scaled_shape) {
                            let scale_y = match shape {
                                Shape::Rectangle(_, h_3) => (e.pos.y + h_1 - (pos.y + h_3)) / t_1,
                                _ => 0.0,
                            };
                            scaled_pos.y = e.scaled_pos.y + h_2 - t_2 * scale_y
                        } else {
                            scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
                        }
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                    }
                }
                return;
            }
        }
        match scale_option {
            Scale::SquareY => {
                match shape {
                    Shape::Rectangle(w, _) => {
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x + w * (scale.x - scale.y) / 2.0;
                    }
                    Shape::RectangleOutline(w, _, _) => {
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x + w * (scale.x - scale.y) / 2.0;
                    }
                    Shape::Line(end_x, _, _) => {
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x + (end_x - pos.x) * (scale.x - scale.y) / 2.0;
                    }
                    _ => {
                        scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                    }
                }
                scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
            }
            _ => {
                scaled_pos.x = layout.scaled_pos.x + (pos.x - layout.pos.x) * scale.x;
                scaled_pos.y = layout.scaled_pos.y + (pos.y - layout.pos.y) * scale.y;
            }
        }
    }

    fn scale_shape(&self, layout: &Layout, scale_option: Scale, shape: Shape, scaled_shape: &mut Shape, pos: Vec2, scaled_pos: Vec2, scale: Vec2) {
        match scale_option {
            Scale::SquareY => {
                match (scaled_shape, shape) {
                    (Shape::Line(end1_x, end1_y, _), Shape::Line(end2_x, end2_y, _)) => {
                        *end1_x = scaled_pos.x + (end2_x - pos.x) * scale.y;
                        *end1_y = scaled_pos.y + (end2_y - pos.y) * scale.y;
                    }
                    (Shape::Rectangle(dim1_x, dim1_y), Shape::Rectangle(dim2_x, dim2_y)) => {
                        *dim1_x = dim2_x * scale.y;
                        *dim1_y = dim2_y * scale.y;
                    }
                    (Shape::RectangleOutline(dim1_x, dim1_y, b1), Shape::RectangleOutline(dim2_x, dim2_y, b2)) => {
                        *dim1_x = dim2_x * scale.y;
                        *dim1_y = dim2_y * scale.y;
                        *b1 = b2 * scale.y;
                    }
                    (Shape::Circle(rad1), Shape::Circle(rad2)) => {
                        *rad1 = rad2 * scale.y;
                    }
                    (Shape::Text(_, font_size1, allign1), Shape::Text(_, font_size2, allign2)) => {
                        *font_size1 = (font_size2 as f32 * scale.y) as i32;
                        match (allign1, allign2) {
                            (Allignment::BoundedLeft(width1), Allignment::BoundedLeft(width2)) => {
                                *width1 = (width2 as f32 * scale.y) as i32;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                match (scaled_shape, shape) {
                    (Shape::Line(end1_x, end1_y, _), Shape::Line(end2_x, end2_y, _)) => {
                        *end1_x = layout.scaled_pos.x + (end2_x - layout.pos.x) * scale.x;
                        *end1_y = layout.scaled_pos.y + (end2_y - layout.pos.y) * scale.y;
                    }
                    (Shape::Rectangle(dim1_x, dim1_y), Shape::Rectangle(dim2_x, dim2_y)) => {
                        *dim1_x = dim2_x * scale.x;
                        *dim1_y = dim2_y * scale.y;
                    }
                    (Shape::RectangleOutline(dim1_x, dim1_y, b1), Shape::RectangleOutline(dim2_x, dim2_y, b2)) => {
                        *dim1_x = dim2_x * scale.x;
                        *dim1_y = dim2_y * scale.y;
                        *b1 = b2 * scale.y;
                    }
                    (Shape::Circle(rad1), Shape::Circle(rad2)) => {
                        *rad1 = rad2 * scale.y;
                    }
                    (Shape::Text(_, font_size1, allign1), Shape::Text(_, font_size2, allign2)) => {
                        *font_size1 = (font_size2 as f32 * scale.y) as i32;
                        match (allign1, allign2) {
                            (Allignment::BoundedLeft(width1), Allignment::BoundedLeft(width2)) => {
                                *width1 = (width2 as f32 * scale.x) as i32;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn scale_from_parent(&self, element: &Element, component: &Rc<RefCell<Element>>) {
        let mut c = component.borrow_mut();
        if let (Shape::Rectangle(w, h), Shape::Rectangle(w_scale, h_scale)) = (&element.shape, &element.scaled_shape) {
            let scale = Vec2::new(*w_scale / *w, *h_scale / *h);
            c.scaled_pos.x = (c.pos.x - element.pos.x) * scale.x + element.scaled_pos.x;
            c.scaled_pos.y = (c.pos.y - element.pos.y) * scale.y + element.scaled_pos.y;

            let shape = c.shape.clone();
            match (&mut c.scaled_shape, shape) {
                (Shape::Rectangle(dim1_x, dim1_y), Shape::Rectangle(dim2_x, dim2_y)) => {
                    *dim1_x = dim2_x * scale.y;
                    *dim1_y = dim2_y * scale.y;
                }
                (Shape::RectangleOutline(dim1_x, dim1_y, b1), Shape::RectangleOutline(dim2_x, dim2_y, b2)) => {
                    *dim1_x = dim2_x * scale.x;
                    *dim1_y = dim2_y * scale.y;
                    *b1 = b2 * scale.y;
                }
                (Shape::Line(end1_x, end1_y, _), Shape::Line(end2_x, end2_y, _)) => {
                    *end1_x = element.scaled_pos.x + (end2_x - element.pos.x) * scale.x;
                    *end1_y = element.scaled_pos.y + (end2_y - element.pos.y) * scale.y;
                }
                (Shape::Circle(rad1), Shape::Circle(rad2)) => {
                    *rad1 = rad2 * scale.y;
                }
                (Shape::Text(_, font_size1, allign1), Shape::Text(_, font_size2, allign2)) => {
                    *font_size1 = (font_size2 as f32 * scale.y) as i32;
                    match (allign1, allign2) {
                        (Allignment::BoundedLeft(width1), Allignment::BoundedLeft(width2)) => {
                            *width1 = (width2 as f32 * scale.x) as i32;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }        
    
            for other_c in &c.components {
                self.scale_from_parent(element, other_c);
            }
        }
    }

    pub fn fetch_element(&self, id: usize) -> Option<Ref<Element>> {
        if let Some(index) = self.element_map.get(&id) {
            if let Some(element) = self.elements.get(*index) {
                return Some(element.borrow());
            }
        }
        None
    }

    pub fn fetch_element_mut(&self, id: usize) -> Option<RefMut<Element>> {
        if let Some(index) = self.element_map.get(&id) {
            if let Some(element) = self.elements.get(*index) {
                return Some(element.borrow_mut());
            }
        }
        None
    }

    pub fn link_layout(&mut self) {
        for manager in &mut self.screens {
            manager.link_layout();
        }
    }

    pub fn begin_immediate(&mut self, current_width: f32, current_height: f32) {
        for screen_index in self.current_screens.clone() {
            self.scale_layout(screen_index, current_width, current_height);
        }

        self.element_map.clear();
        self.elements.clear();
        for screen in &mut self.screens {
            screen.layout.clear();
            screen.layout_map.clear();
        }
    }
    
    pub fn begin_immediate_retain_layout(&mut self, current_width: f32, current_height: f32) {
        for screen_index in self.current_screens.clone() {
            self.scale_layout(screen_index, current_width, current_height);
        }

        self.element_map.clear();
        self.elements.clear();
        for screen in &mut self.screens {
            let mut new_layout: Vec<Rc<RefCell<Layout>>> = Vec::new();
            let mut new_layout_map: HashMap<String, usize> = HashMap::new();
            for (name, index) in screen.layout_map.iter() {
                if let Some(layout) = screen.layout.get(*index) {
                    let mut l = layout.borrow_mut();
                    l.components.clear();
                    l.children.clear();
                    if !l.sub_layout {
                        let new_index = new_layout.len();
                        new_layout.push(Rc::clone(layout));
                        new_layout_map.insert(name.to_string(), new_index);
                    }
                }
            }
        }
    }

    pub fn end_immediate(&mut self, handle: &mut RainHandle) {
        self.draw(handle);
    }

    pub fn draw(&mut self, handle: &mut RainHandle) {
        rain_draw(handle, &mut self.pending_draw_calls);
    }

    pub fn load_draw_calls(&mut self) {
        let mut draw_calls: VecDeque<DrawCall> = VecDeque::new();
        for screen in &self.current_screens {
            if let Some(manager) = self.screens.get(*screen) {
                self.load_draw_calls_layout(manager, &mut draw_calls);
            }
        }
        self.pending_draw_calls = draw_calls;
    }

    pub fn load_draw_calls_layout(&self, manager: &LayoutManager, draw_calls: &mut VecDeque<DrawCall>) {
        for layout in &manager.layout {
            let l = layout.borrow();
            for component in &l.components {
                self.load_draw_calls_recursive(&l, component, draw_calls);
            }
        }
    }

    pub fn load_draw_calls_recursive(&self, layout: &Layout, element: &Rc<RefCell<Element>>, draw_calls: &mut VecDeque<DrawCall>) {
        let e = element.borrow();
        if e.scissor {
            if let Shape::Rectangle(width, height) = e.scaled_shape {
                draw_calls.push_back(DrawCall::option(DrawOption::BeginScissorMode(e.scaled_pos.x as i32, e.scaled_pos.y as i32, width as i32, height as i32)));
                // RaylibScissorModeExt::draw_scissor_mode(d, e.scaled_pos.x as i32, e.scaled_pos.y as i32, width as i32, height as i32, 
                //     |mut s| {
                //         e.draw(&mut s);
                //         for component in &e.components {
                //             self.load_draw_calls_recursive(&mut s, layout, component, draw_calls);
                //         }
                //     }
                // );
                // return;
            }
        }
        if let Some(call) = e.get_draw_call() {
            draw_calls.push_back(call);
        }
        for component in &e.components {
            self.load_draw_calls_recursive(layout, component, draw_calls);
        }
        if e.scissor {
            draw_calls.push_back(DrawCall::option(DrawOption::EndScissorMode));
        }
    }

    pub fn update_collision(&self, point: Vec2) {
        for screen in &self.current_screens {
            if let Some(manager) = self.screens.get(*screen) {
                manager.check_collision(point);
            }
        }
    }

    pub fn set_draw(&self, id: usize, draw: bool) {
        if let Some(index) = self.element_map.get(&id) {
            if let Some(element) = self.elements.get(*index) {
                let mut e = element.borrow_mut();
                e._draw = draw;
            }
        }
    }

    pub fn set_draw_recursive(&self, id: usize, draw: bool) {
        if let Some(index) = self.element_map.get(&id) {
            if let Some(element) = self.elements.get(*index) {
                self.set_draw_recursive_element(element, draw);
            }
        }
    }

    fn set_draw_recursive_element(&self, element: &Rc<RefCell<Element>>, draw: bool) {
        let mut e = element.borrow_mut();
        e._draw = draw;
        for component in &e.components {
            self.set_draw_recursive_element(component, draw);
        }
    }

    pub fn fetch_texture(&self, name: &str) -> Arc<Texture> {
        Arc::clone(self.textures.get(name).unwrap())
    }

    pub fn borrow_layout(&self, name: &str) -> Option<Ref<Layout>> {
        if let Some(mananager) = self.screens.get(self.current_screen) {
            return mananager.borrow_layout(name);
        }
        None
    }

    pub fn begin_scissor(&mut self, x: f32, y: f32, width: f32, height: f32, scale: Scale, layout_name: &str, name: &str) {
        let mut draw_calls: VecDeque<DrawCall> = VecDeque::new();
        if let Some(mananager) = self.screens.get_mut(self.current_screen) {
            if let Some(layout) = mananager.fetch_layout(layout_name) {
                let mut l = layout.borrow_mut();
                let new_layout = Rc::new(RefCell::new(Layout::sub_layout(
                    Vec2::new(x, y), Vec2::new(width, height), scale, false, false
                )));
                let index = mananager.layout.len();
                l.children.push(index);
                mananager.layout.push(Rc::clone(&new_layout));
                mananager.layout_map.insert(name.to_string(), index);
                let mut n = new_layout.borrow_mut();
                mananager.scale_sub_layout(&l, &mut n);
                
                draw_calls.push_back(DrawCall::option(DrawOption::BeginScissorMode(
                    n.scaled_pos.x as i32, n.scaled_pos.y as i32, n.scaled_dim.x as i32, n.scaled_dim.y as i32
                )));
            }
        }
        self.pending_draw_calls.extend(draw_calls);
    }

    pub fn end_scissor(&mut self) {
        self.pending_draw_calls.push_back(DrawCall::option(DrawOption::EndScissorMode));
    }
}