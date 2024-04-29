use serde::Deserialize;
use simsearch::SearchOptions;
use simsearch::SimSearch;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Node {
    pub input: HashMap<String, String>,
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
        format!(
            "{} {} {} {}",
            self.display_name, self.name, self.description, self.category
        )
    }
}

pub fn nodes() -> HashMap<String, Node> {
    let url = "http://127.0.0.1:8188/object_info";

    let response = reqwest::blocking::get(url).expect("Failed to make GET request");

    assert!(response.status().is_success());

    response.json().expect("Failed to deserialize JSON")

    // [
    //     (
    //         "A".into(),
    //         Node {
    //             input: [].into(),
    //             output_is_list: vec![false, false],
    //             output_name: vec!["Text".into(), "Image".into()],
    //             name: "A".into(),
    //             display_name: "A".into(),
    //             output: vec!["TXT".into(), "IMG".into()],
    //             description: "Node of type A".into(),
    //             category: "Dummy".into(),
    //             output_node: false,
    //         },
    //     ),
    //     (
    //         "B".into(),
    //         Node {
    //             input: [("Text".into(), "TXT".into())].into(),
    //             output_is_list: vec![],
    //             output_name: vec![],
    //             name: "B".into(),
    //             display_name: "B".into(),
    //             output: vec![],
    //             description: "Node of type B".into(),
    //             category: "Dummy".into(),
    //             output_node: false,
    //         },
    //     ),
    // ]
    // .into()
}

pub fn build_index(nodes: &HashMap<String, Node>) -> SimSearch<String> {
    let mut index = SimSearch::new_with(SearchOptions::new().threshold(0.65).levenshtein(false));

    for (k, v) in nodes {
        index.insert(k.clone(), &v.index());
    }

    index
}
