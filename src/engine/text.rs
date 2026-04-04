use std::collections::HashMap;

pub const DEFAULT_FONT_SIZE: f32 = 30.0;
pub const DEFAULT_LINE_HEIGHT: f32 = 42.0;

pub struct TextState {
    pub font_system: glyphon::FontSystem,
    pub swash_cache: glyphon::SwashCache,
    pub viewport: glyphon::Viewport,
    pub atlas: glyphon::TextAtlas,
    pub renderer: glyphon::TextRenderer,
    pub buffer_pool: TextBufferPool,
    pub to_draw: Vec<TextInfo>,
}

impl TextState {
    pub fn measure_text(&mut self, text: &str, font_size: u32) -> f32 {
        self.buffer_pool.measure_text(text, font_size, &mut self.font_system)
    }
}

pub struct TextBufferPool {
    pub available: HashMap<u32, Vec<glyphon::Buffer>>,
    pub using: Vec<glyphon::Buffer>,
    capacity: usize,
}

impl TextBufferPool {
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
            using: Vec::new(),
            capacity: 0,
        }
    }

    pub fn reset(&mut self) {
        for buffer in self.using.drain(..) {
            let size = buffer.metrics().font_size as u32;
            self.available.entry(size).or_default().push(buffer);
        }
    }

    pub fn set_capacity(&mut self, font_system: &mut glyphon::FontSystem, capacity: usize) {
        if capacity < self.capacity {
            let mut to_remove = self.capacity - capacity;
            self.reset();
            let mut keys: Vec<u32> = self.available.keys().copied().collect();
            keys.sort();
            for key in keys.iter().rev() {
                let mut entry = self.available.remove(key).unwrap();
                let length = entry.len();
                if length > to_remove {
                    entry.truncate(to_remove - length);
                    self.available.insert(key.clone(), entry);
                    break;
                } else {
                    to_remove -= length;
                }  
            }
        } else {
            let to_add = capacity - self.capacity;
            let buffer = vec![glyphon::Buffer::new(font_system, glyphon::Metrics::new(DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT)); to_add];
            self.available.entry(DEFAULT_FONT_SIZE as u32).or_default().extend(buffer);
        }
        self.capacity = capacity;
    }

    pub fn resize(&mut self, font_system: &mut glyphon::FontSystem) {
        self.set_capacity(font_system, self.capacity * 2);
    }

    fn fetch_default_buffer(&mut self, font_system: &mut glyphon::FontSystem) -> glyphon::Buffer {
        let mut default_buffer = self.available.remove(&(DEFAULT_FONT_SIZE as u32)).unwrap_or_default();
        if let Some(b) = default_buffer.pop() {
            self.available.insert(DEFAULT_FONT_SIZE as u32, default_buffer);
            b
        } else {
            self.resize(font_system);
            self.available.entry(DEFAULT_FONT_SIZE as u32).or_default().pop().unwrap()
        }
    }

    pub fn add_text(&mut self, text: &str, font_size: u32, font_system: &mut glyphon::FontSystem) -> usize {
        let (mut buffer, i) = if let Some(buffer_entry) = self.available.get_mut(&font_size) {
            if let Some(b) = buffer_entry.pop() {
                (b, font_size)
            } else {
                (self.fetch_default_buffer(font_system), DEFAULT_FONT_SIZE as u32)
            }
        } else {
            (self.fetch_default_buffer(font_system), DEFAULT_FONT_SIZE as u32)
        };

        buffer.set_text(font_system, text, &glyphon::Attrs::new(), glyphon::Shaping::Basic);
        if i != font_size {
            buffer.set_metrics(font_system, glyphon::Metrics { font_size: font_size as f32, line_height: font_size as f32 * 6.2 });
        }

        let index = self.using.len();
        self.using.push(buffer);
        index
    }

    pub fn measure_text(&mut self, text: &str, font_size: u32, font_system: &mut glyphon::FontSystem) -> f32 {
        let (mut buffer, i) = if let Some(buffer_entry) = self.available.get_mut(&font_size) {
            if let Some(b) = buffer_entry.pop() {
                (b, font_size)
            } else {
                (self.fetch_default_buffer(font_system), DEFAULT_FONT_SIZE as u32)
            }
        } else {
            (self.fetch_default_buffer(font_system), DEFAULT_FONT_SIZE as u32)
        };

        buffer.set_text(font_system, text, &glyphon::Attrs::new(), glyphon::Shaping::Basic);
        if i != font_size {
            buffer.set_metrics(font_system, glyphon::Metrics { font_size: font_size as f32, line_height: DEFAULT_LINE_HEIGHT });
        }

        let width = buffer.layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, |acc, x| acc + x);

        self.available.entry(i).or_default().push(buffer);

        width
    }
}

pub struct TextInfo {
    pub buffer_index: usize,
    pub x: f32,
    pub y: f32,
    pub color: glyphon::Color,
}