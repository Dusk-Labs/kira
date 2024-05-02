use serde::Deserialize;
use std::{collections::HashMap, error::Error};

pub struct Backend {}
impl Backend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend {
    pub fn query_available_nodes(&self) -> Result<HashMap<String, Node>, Box<dyn Error>> {
        let url = "http://127.0.0.1:8188/object_info";

        let response = reqwest::blocking::get(url)?;

        assert!(response.status().is_success());

        Ok(response.json().expect("Failed to deserialize JSON"))
    }
}

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
