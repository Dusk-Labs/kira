use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use simsearch::SimSearch;
use simsearch::SearchOptions;

pub type Nodes = HashMap<String, Node>;

#[derive(Deserialize, Debug)]
pub struct Node {
    pub input: HashMap<String, Value>,
    pub output: Vec<String>,
    pub output_is_list: Vec<bool>,
    pub output_name: Vec<String>,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub output_node: bool,
}

impl Node {
    pub fn index(&self) -> String {
        format!("{} {} {} {}", self.display_name, self.name, self.description, self.category)
    }
}

pub fn nodes() -> Nodes {
    let url = "http://127.0.0.1:8188/object_info";

    let response = reqwest::blocking::get(url).expect("Failed to make GET request");

    assert!(response.status().is_success());

    response.json().expect("Failed to deserialize JSON")
}

pub fn build_index(nodes: &Nodes) -> SimSearch<String> {
    let mut index = SimSearch::new_with(SearchOptions::new().threshold(0.65).levenshtein(false));

    for (k, v) in nodes {
        index.insert(k.clone(), &v.index());
    }

    index
}
