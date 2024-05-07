use serde::{Deserialize, Serialize};
use slint::SharedString;

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<NodeInstance>,
    links: Vec<Link>,
    zoom: f32,
    offset: (f32, f32),
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            links: vec![],
            zoom: 2.,
            offset: (0., 0.),
        }
    }
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
    pub fn offset(&self) -> (f32, f32) {
        self.offset
    }
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }
    pub fn set_offset(&mut self, offset: (f32, f32)) {
        self.offset = offset;
    }
    pub fn add_node(&mut self, id: NodeType) {
        self.nodes.push(NodeInstance {
            ty: id,
            pos: (20. - self.offset.0, 20. - self.offset.1),
        });
    }
    pub fn set_node_position(&mut self, node_idx: usize, x: f32, y: f32) {
        let node_ref = self.nodes.get_mut(node_idx).unwrap();
        node_ref.pos.0 = x;
        node_ref.pos.1 = y;
    }
    pub fn remove_link(&mut self, idx: usize) {
        self.links.remove(idx);
    }
    pub fn add_link(&mut self, lnk: Link) {
        self.links.push(lnk);
    }
    pub fn get_nodes(&self) -> &[NodeInstance] {
        &self.nodes
    }
    pub fn get_node(&self, idx: usize) -> Option<&NodeInstance> {
        self.nodes.get(idx)
    }
    pub fn get_links(&self) -> &[Link] {
        &self.links
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInstance {
    pub ty: NodeType,
    pub pos: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub src_node: usize,
    pub src_slot: usize,
    pub dst_node: usize,
    pub dst_slot: usize,
    pub ty: LinkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkType(pub String);

impl From<LinkType> for SharedString {
    fn from(value: LinkType) -> Self {
        value.0.into()
    }
}
impl From<&str> for LinkType {
    fn from(value: &str) -> Self {
        LinkType(value.into())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeType(pub String);

impl From<&str> for NodeType {
    fn from(value: &str) -> Self {
        NodeType(value.into())
    }
}
impl From<String> for NodeType {
    fn from(value: String) -> Self {
        NodeType(value)
    }
}
impl From<SharedString> for NodeType {
    fn from(value: SharedString) -> Self {
        NodeType(value.into())
    }
}
