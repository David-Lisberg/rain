use serde::{Deserialize, Serialize};
use glam::Vec2;
use rain::engine::color::Color;

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read};
use std::io::Write;

pub struct Tile;
#[derive(Serialize, Deserialize)]
pub struct Vec2JSON {
    pub x: f32,
    pub y: f32,
}
#[derive(Serialize, Deserialize)]
pub struct TileJSON {
    pub position: Vec2JSON,
    pub size: Vec2JSON,
    pub color: Color,
}

pub fn write_tile() {
    let tile = TileJSON {
        position: Vec2JSON { x: 0.0, y: 0.0 },
        size: Vec2JSON { x: 1.0, y: 1.0 },
        color: Color::BLACK
    };
    let serialized = serde_json::to_string(&tile).unwrap();
    let path = String::from("res/saves/test.txt");
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Error opening file.");
    writeln!(&mut file, "{}", serialized).expect("Error writing tile.");
}