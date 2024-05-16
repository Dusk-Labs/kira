pub mod node;

use std::collections::HashMap;
use std::error::Error;

use node::Node;
use node::RawNode;

#[derive(Debug)]
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

        let text = response.text().expect("API returned no body.");
        let mut de = serde_json::Deserializer::from_str(&text);
        let result: HashMap<String, RawNode> = match serde_path_to_error::deserialize(&mut de) {
            Ok(x) => x,
            Err(err) => {
                panic!("Failed to deserialize body: {:?}", err.path().to_string());
            }
        };

        let result = result.into_iter().map(|(k, v)| (k, Node::from(v))).collect::<HashMap<_, _>>();

        Ok(result)
    }
}
