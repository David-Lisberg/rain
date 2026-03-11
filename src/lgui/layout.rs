use std::cell::{Ref, RefCell};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use glam::Vec2;

use super::element::{check_collision, Element, Scale};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
}

pub struct Layout {
    pub pos: Vec2,
    pub scaled_pos: Vec2,
    pub dim: Vec2,
    pub scaled_dim: Vec2,
    pub scale: Scale,
    pub bordered_above: Vec<(usize, Axis)>,
    pub bordered_below: Vec<(usize, Axis)>,
    pub components: Vec<Rc<RefCell<Element>>>,
    pub children: Vec<usize>,
    pub sub_layout: bool,
    pub scissor: bool,
    pub recursive: bool,
}

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Layout(pos({},{}),dim({},{}))", self.pos.x, self.pos.y, self.dim.x, self.dim.y)
    }
}

impl Layout {
    pub fn new(pos: Vec2, dim: Vec2, scale: Scale) -> Self {
        Self {
            pos: pos,
            scaled_pos: pos,
            dim: dim,
            scaled_dim: dim,
            scale: scale,
            bordered_above: Vec::new(),
            bordered_below: Vec::new(),
            components: Vec::new(),
            children: Vec::new(),
            sub_layout: false,
            scissor: false,
            recursive: false,
        }
    }

    pub fn sub_layout(pos: Vec2, dim: Vec2, scale: Scale, scissor: bool, recursive: bool) -> Self {
        Self {
            pos: pos,
            scaled_pos: pos,
            dim: dim,
            scaled_dim: dim,
            scale: scale,
            bordered_above: Vec::new(),
            bordered_below: Vec::new(),
            components: Vec::new(),
            children: Vec::new(),
            sub_layout: true,
            scissor: scissor,
            recursive: recursive,
        }
    }
}


pub struct LayoutManager {
    pub layout: Vec<Rc<RefCell<Layout>>>,
    pub layout_map: HashMap<String, usize>,
}

impl LayoutManager {
    pub fn new(screen_width: f32, screen_height: f32, scale: Scale) -> Self {
        let root = Rc::new(RefCell::new(
            Layout::new(Vec2::new(0.0, 0.0), Vec2::new(screen_width, screen_height), scale)
        ));
        let mut layout = Vec::new();
        layout.push(root);
        
        let mut layout_map: HashMap<String, usize> = HashMap::new();
        layout_map.insert("root".to_string(), 0);

        LayoutManager {
            layout: layout,
            layout_map: layout_map,
        }
    }

     pub fn split(&mut self, name: &str, scale: (Scale, Scale), new_name: (&str, &str), split: f32, axis: Axis) {
        if let Some(layout) = hash_map_link_remove(&name.to_string(), &mut self.layout_map, &mut self.layout) {
            let (layout1, layout2) = split_layout(&layout, scale, split, axis);

            let i1 = self.layout.len();
            self.layout.push(Rc::clone(&layout1));
            self.layout_map.insert(new_name.0.to_string(), i1);
            let i2 = self.layout.len();
            self.layout.push(Rc::clone(&layout2));
            self.layout_map.insert(new_name.1.to_string(), i2);
        }
    }

    pub fn find_root(&self) -> Option<usize> {
        let mut root: Option<usize> = None;
        for (i, layout) in self.layout.iter().enumerate() {
            let l = layout.borrow();
            if l.pos == Vec2::ZERO {
                root = Some(i);
                break;
            }
        }
        root
    }

    pub fn fetch_layout(&self, name: &str) -> Option<Rc<RefCell<Layout>>> {
        if let Some(index) = self.layout_map.get(name) {
            if let Some(layout) = self.layout.get(*index) {
                return Some(Rc::clone(layout));
            }
        }
        None
    }

    pub fn borrow_layout(&self, name: &str) -> Option<Ref<Layout>> {
        if let Some(index) = self.layout_map.get(name) {
            if let Some(layout) = self.layout.get(*index) {
                return Some(layout.borrow());
            }
        }
        None
    }

    pub fn link_layout(&mut self) {
        for current in &self.layout {
            let mut c_borrow = current.borrow_mut();
            if c_borrow.sub_layout {
                continue;
            }
            c_borrow.bordered_above.clear();
            c_borrow.bordered_below.clear();
            for (i, other) in self.layout.iter().enumerate() {
                if !Rc::ptr_eq(current, other) {
                    let o_borrow = other.borrow();
                    handle_bordered(&mut c_borrow, &o_borrow,  i);
                }
            }
        }
    }

    pub fn scale_from_root(&mut self, scaled_dim: &Vec2, screen_dim: &Vec2) {
        let scale = Vec2::new(scaled_dim.x / screen_dim.x, scaled_dim.y / screen_dim.y);
        if let Some(root) = self.find_root() {
            self.scale_recursive(&scale, screen_dim, root);
        }
    }

    fn scale_recursive(&mut self, scale: &Vec2, screen_dim: &Vec2, start: usize) {
        let mut searched: Vec<usize> = Vec::new();
        let mut to_search: VecDeque<usize> = VecDeque::new();
        to_search.push_back(start);
        while let Some(index) = to_search.pop_front() {
            if let Some(layout) = self.layout.get(index) {
                let mut l = layout.borrow_mut();
                match l.scale {
                    Scale::XY => {
                        l.scaled_pos.x = l.pos.x * scale.x;
                        l.scaled_pos.y = l.pos.y * scale.y;
                        l.scaled_dim.x = l.dim.x * scale.x;
                        l.scaled_dim.y = l.dim.y * scale.y;
                    }
                    Scale::X => {
                        l.scaled_pos.x = l.pos.x * scale.x;
                        l.scaled_dim.x = l.dim.x * scale.x;
                    }
                    Scale::Y => {
                        l.scaled_pos.y = l.pos.y * scale.y;
                        l.scaled_dim.y = l.dim.y * scale.y;
                    }
                    Scale::SquareX => {
                        l.scaled_pos.x = l.pos.x * scale.x;
                        l.scaled_pos.y = l.pos.y * scale.x;
                        l.scaled_dim.x = l.dim.x * scale.x;
                        l.scaled_dim.y = l.dim.y * scale.x;
                    }
                    Scale::SquareY => {
                        l.scaled_pos.x = l.pos.x * scale.y;
                        l.scaled_pos.y = l.pos.y * scale.y;
                        l.scaled_dim.x = l.dim.x * scale.y;
                        l.scaled_dim.y = l.dim.y * scale.y;
                    }
                    Scale::LessW => {
                        if scale.x < 1.0 {
                            l.scaled_pos.x = l.pos.x * scale.x;
                            l.scaled_pos.y = l.pos.y * scale.x;
                            l.scaled_dim.x = l.dim.x * scale.x;
                            l.scaled_dim.y = l.dim.y * scale.x;
                        } else {
                            l.scaled_pos.x = l.pos.x * scale.y;
                            l.scaled_pos.y = l.pos.y * scale.y;
                            l.scaled_dim.x = l.dim.x * scale.y;
                            l.scaled_dim.y = l.dim.y * scale.y;
                        }
                    }
                    Scale::LessH => {
                        if scale.y < 1.0 {
                            l.scaled_pos.x = l.pos.x * scale.y;
                            l.scaled_pos.y = l.pos.y * scale.y;
                            l.scaled_dim.x = l.dim.x * scale.y;
                            l.scaled_dim.y = l.dim.y * scale.y;
                        } else {
                            l.scaled_pos.x = l.pos.x * scale.x;
                            l.scaled_pos.y = l.pos.y * scale.x;
                            l.scaled_dim.x = l.dim.x * scale.x;
                            l.scaled_dim.y = l.dim.y * scale.x;
                        }
                    }
                    Scale::None => {}
                }
                for (b_index, axis) in &l.bordered_below {
                    if let Some(below) = self.layout.get(*b_index) {
                        let mut b = below.borrow_mut();
                        match axis {
                            Axis::X => self.adjust_below(&mut b, l.scaled_pos.x + l.scaled_dim.x, *axis, index, true),
                            Axis::Y => self.adjust_below(&mut b, l.scaled_pos.y + l.scaled_dim.y, *axis, index, true),
                        }
                    }
                }
                for (a_index, axis) in &l.bordered_above {
                    if let Some(above) = self.layout.get(*a_index) {
                        let mut a = above.borrow_mut();
                        match axis {
                            Axis::X => self.adjust_above(&mut a, l.scaled_pos.x, *axis, index, true),
                            Axis::Y => self.adjust_above(&mut a, l.scaled_pos.y, *axis, index, true),
                        }
                    }
                }
                if l.pos.x + l.dim.x == screen_dim.x {
                    let diff = screen_dim.x * scale.x - (l.scaled_pos.x + l.scaled_dim.x);
                    l.scaled_dim.x += diff;
                }
                if l.pos.y + l.dim.y == screen_dim.y {
                    let diff = screen_dim.y * scale.y - (l.scaled_pos.y + l.scaled_dim.y);
                    l.scaled_dim.y += diff;
                }
                for (b_index, _) in &l.bordered_below {
                    if !searched.contains(b_index) {
                        to_search.push_back(*b_index);
                    }
                }
                searched.push(index);
            }
        }
        let mut to_search: VecDeque<(usize, usize)> = VecDeque::new();
        for (i, layout) in self.layout.iter().enumerate() {
            let l = layout.borrow();
            if !l.sub_layout {
                for child in &l.children {
                    to_search.push_back((i, *child));
                }
            }
        }
        while let Some((parent_index, child_index)) = to_search.pop_front() {
            let parent = self.layout.get(parent_index).unwrap();
            let child = self.layout.get(child_index).unwrap();
            let p = parent.borrow();
            let mut c = child.borrow_mut();
            self.scale_sub_layout(&p, &mut c);
        }
    }

    pub fn scale_sub_layout(&self, parent: &Layout, layout: &mut Layout) {
        let scale = Vec2::new(parent.scaled_dim.x / parent.dim.x, parent.scaled_dim.y / parent.dim.y);
        let scale_option = if layout.recursive {
            Scale::None
        } else {
            layout.scale.clone()
        };
        match scale_option {
            Scale::SquareY => {
                layout.scaled_pos.x = parent.scaled_pos.x + (layout.pos.x - parent.pos.x) * scale.x + layout.dim.x * (scale.x - scale.y) / 2.0;
                layout.scaled_pos.y = parent.scaled_pos.y + (layout.pos.y - parent.pos.y) * scale.y;
                layout.scaled_dim.x = layout.dim.x * scale.y;
                layout.scaled_dim.y = layout.dim.y * scale.y;
            }
            _ => {
                layout.scaled_pos.x = parent.scaled_pos.x + (layout.pos.x - parent.pos.x) * scale.x;
                layout.scaled_pos.y = parent.scaled_pos.y + (layout.pos.y - parent.pos.y) * scale.y;
                layout.scaled_dim.x = layout.dim.x * scale.x;
                layout.scaled_dim.y = layout.dim.y * scale.y;
            }
        }
    }

    fn adjust_above(&self, layout: &mut Layout, edge: f32, axis: Axis, index: usize, recursive: bool) {
        match axis {
            Axis::X => {
                let diff = edge - (layout.scaled_pos.x + layout.scaled_dim.x);
                layout.scaled_dim.x += diff;
            }
            Axis::Y => {
                let diff = edge - (layout.scaled_pos.y + layout.scaled_dim.y);
                layout.scaled_dim.y += diff;
            }
        }
        if recursive {
            for (b_index, a) in &layout.bordered_below {
                if index == *b_index {
                    continue;
                }
                if axis == *a {
                    if let Some(other) = self.layout.get(*b_index) {
                        let mut o = other.borrow_mut();
                        self.adjust_below(&mut o, edge, axis, index, false);
                    }
                }
            }
        }
    }

    fn adjust_below(&self, layout: &mut Layout, edge: f32, axis: Axis, index: usize, recursive: bool) {
        match axis {
            Axis::X => {
                let diff = edge - layout.scaled_pos.x;
                layout.scaled_pos.x += diff;
            }
            Axis::Y => {
                let diff = edge - layout.scaled_pos.y;
                layout.scaled_pos.y += diff;
            }
        }
        if recursive {
            for (a_index, a) in &layout.bordered_above {
                if index == *a_index {
                    continue;
                }
                if axis == *a {
                    if let Some(other) = self.layout.get(*a_index) {
                        let mut o = other.borrow_mut();
                        self.adjust_above(&mut o, edge, axis, index, false);
                    }
                }
            }
        }
    }

    pub fn check_collision(&self, point: Vec2) {
        for layout in &self.layout {
            let l = layout.borrow();
            for component in &l.components {
                self.check_collision_recursive(component, point);
            }
        }
    }

    pub fn check_collision_recursive(&self, element: &Rc<RefCell<Element>>, point: Vec2) {
        let mut e = element.borrow_mut();
        if e.check_collision {
            if let Some(collider) = &e.collider {
                e.collided = check_collision(&collider.scaled_pos, &collider.scaled_shape, point);
            } else {
                e.collided = check_collision(&e.scaled_pos, &e.scaled_shape, point);
            }
            if e.collided {
                // println!("id: {}", e.id);
            }
        }
        for component in &e.components {
            self.check_collision_recursive(component, point);
        }
    }
}

fn split_layout(layout: &Rc<RefCell<Layout>>, scale: (Scale, Scale), split: f32, axis: Axis) -> (Rc<RefCell<Layout>>, Rc<RefCell<Layout>>) {
    let l_b = layout.borrow();
    let (layout1, layout2) = match axis {
        Axis::X => {
            (Rc::new(RefCell::new(Layout::new(Vec2::new(l_b.pos.x, l_b.pos.y), Vec2::new(split, l_b.dim.y), scale.0))),
            Rc::new(RefCell::new(Layout::new(Vec2::new(l_b.pos.x + split, l_b.pos.y), Vec2::new(l_b.dim.x - split, l_b.dim.y), scale.1))))
        }
        Axis::Y => {
            (Rc::new(RefCell::new(Layout::new(Vec2::new(l_b.pos.x, l_b.pos.y), Vec2::new(l_b.dim.x, split), scale.0))),
            Rc::new(RefCell::new(Layout::new(Vec2::new(l_b.pos.x, l_b.pos.y + split), Vec2::new(l_b.dim.x, l_b.dim.y - split), scale.1))))
        }
    };

    (layout1, layout2)
}

fn handle_bordered(l1: &mut Layout, l2: &Layout, index: usize) {
        if l1.pos.x == l2.pos.x + l2.dim.x && l2.pos.y <= l1.pos.y + l1.dim.y && l2.pos.y + l2.dim.y >= l1.pos.y {
            l1.bordered_above.push((index, Axis::X));
        }
        if l1.pos.y == l2.pos.y + l2.dim.y && l2.pos.x <= l1.pos.x + l1.dim.x && l2.pos.x + l2.dim.x >= l1.pos.x {
            l1.bordered_above.push((index, Axis::Y));
        }
        if l1.pos.x + l1.dim.x == l2.pos.x && l2.pos.y <= l1.pos.y + l1.dim.y && l2.pos.y + l2.dim.y >= l1.pos.y {
            l1.bordered_below.push((index, Axis::X));
        }
        if l1.pos.y + l1.dim.y == l2.pos.y && l2.pos.x <= l1.pos.x + l1.dim.x && l2.pos.x + l2.dim.x >= l1.pos.x {
            l1.bordered_below.push((index, Axis::Y));
        }
}

pub fn hash_map_link_remove<K, V>(id: &K, map: &mut HashMap<K, usize>, vector: &mut Vec<V>) -> Option<V>
where
    K: Hash + std::cmp::Eq
{
    if vector.is_empty() {
        return None;
    }
    if let Some(index) = map.remove(id) {
        let last = vector.len() - 1;

        if index != last {
            vector.swap(index, last);

            if let Some((_, value)) = map.iter_mut().find(|(_, v)| **v == last) {
                *value = index;
            } else {
                return None;
            }
        }   
        
        return vector.pop();
    }
    None
}