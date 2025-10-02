use std::sync::Arc;

use crate::{texture::Texture, vertex::UIVertex};

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u16>,
    pub material: Arc<Texture>,
}