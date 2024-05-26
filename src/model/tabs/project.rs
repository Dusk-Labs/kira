use self::graph::{Graph, LinkType, NodeType};
use simsearch::{SearchOptions, SimSearch};
use std::collections::HashMap;
use crate::model::backend::node::NodeField;

pub mod graph;

#[derive(Debug)]
pub struct Project {
    available_node_index: SimSearch<NodeType>,
    available_nodes: HashMap<NodeType, Node>,
    graph: Graph,
    file_path: Option<String>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            available_nodes: HashMap::new(),
            available_node_index: Self::empty_index(),
            graph: Graph::new(),
            file_path: None,
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
    }

    pub fn set_available_nodes(&mut self, nodes: HashMap<NodeType, Node>) {
        self.available_nodes = nodes;
        self.build_index();
    }

    pub fn available_nodes(&self) -> &HashMap<NodeType, Node> {
        &self.available_nodes
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

    pub fn get_available_node(&self, id: &NodeType) -> Option<Node> {
        self.available_nodes.get(id).cloned()
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
    pub fields: Vec<(String, NodeField)>,
    pub name: String,
    pub description: String,
    pub category: String,
}

impl Node {
    pub fn search_string(&self) -> String {
        format!("{} {} {}", self.name, self.description, self.category)
    }
}
