use simsearch::{SearchOptions, SimSearch};
use slint::SharedString;
use std::collections::HashMap;

pub struct Project {
    available_node_index: SimSearch<NodeType>,
    available_nodes: HashMap<NodeType, Node>,
    nodes: Vec<NodeInstance>,
    links: Vec<Link>,
    subscribers: Vec<Subscriber>,
}

type Subscriber = Box<dyn Fn(&Project)>;

impl Project {
    pub fn new() -> Self {
        Self {
            available_nodes: HashMap::new(),
            available_node_index: Self::empty_index(),
            nodes: vec![],
            links: vec![],
            subscribers: vec![],
        }
    }
    pub fn subscribe(&mut self, f: Subscriber) {
        self.subscribers.push(f);
    }
    fn notify(&self) {
        for s in self.subscribers.iter() {
            s(self);
        }
    }
    pub fn set_available_nodes(&mut self, nodes: HashMap<NodeType, Node>) {
        self.available_nodes = nodes;
        self.build_index();
        self.notify();
    }
    pub fn search_available_nodes(&self, query: &str) -> Vec<(NodeType, Node)> {
        let ids = self.available_node_index.search(query);
        ids.into_iter()
            .take(10)
            .filter_map(|id| {
                self.available_nodes
                    .get_key_value(&id)
                    .map(|(k, v)| (k.clone(), v.clone()))
            })
            .collect()
    }
    pub fn add_node(&mut self, id: NodeType) {
        self.nodes.push(NodeInstance {
            ty: id,
            pos: (0., 0.),
        });
        self.notify();
    }
    pub fn set_node_position(&mut self, node_idx: usize, x: f32, y: f32) {
        let node_ref = self.nodes.get_mut(node_idx).unwrap();
        node_ref.pos.0 = x;
        node_ref.pos.1 = y;
    }
    pub fn remove_link(&mut self, idx: usize) {
        self.links.remove(idx);
        self.notify();
    }
    pub fn add_link(&mut self, lnk: Link) {
        self.links.push(lnk);
        self.notify();
    }
    pub fn get_available_node(&self, id: &NodeType) -> Option<Node> {
        self.available_nodes.get(id).cloned()
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
    fn build_index(&mut self) {
        self.available_node_index = Self::empty_index();

        for (k, v) in self.available_nodes.iter() {
            self.available_node_index
                .insert(k.clone(), &v.search_string());
        }
    }
    fn empty_index() -> SimSearch<NodeType> {
        SimSearch::new_with(SearchOptions::new().threshold(0.65).levenshtein(false))
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub inputs: Vec<(String, LinkType)>,
    pub outputs: Vec<(String, LinkType)>,
    pub name: String,
    pub description: String,
    pub category: String,
}

impl Node {
    pub fn search_string(&self) -> String {
        format!("{} {} {}", self.name, self.description, self.category)
    }
}

pub struct NodeInstance {
    pub ty: NodeType,
    pub pos: (f32, f32),
}

pub struct Link {
    pub src_node: usize,
    pub src_slot: usize,
    pub dst_node: usize,
    pub dst_slot: usize,
    pub ty: LinkType,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
