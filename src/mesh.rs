use crate::vertex::UIVertex;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u16>,
}