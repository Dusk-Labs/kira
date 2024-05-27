pub mod conv;
pub mod node;
pub mod websocket;
pub mod workflow;

use std::collections::HashMap;
use std::error::Error;

use crate::ctrl::Event;
use node::Node;
use node::RawNode;

use tokio::runtime;

#[derive(Debug)]
pub struct Backend {
    rt: runtime::Runtime,
}

impl Backend {
    pub fn new() -> Self {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Self { rt }
    }

    pub fn spawm_client(&self, tx: std::sync::mpsc::Sender<Event>) {
        self.rt.block_on(websocket::WsClient::new(tx).listen());
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

        let result = result
            .into_iter()
            .map(|(k, v)| (k, Node::from(v)))
            .collect::<HashMap<_, _>>();

        Ok(result)
    }

    pub fn exec(compute_graph: &workflow::WorkflowPrompt) {
        let url = "http://127.0.0.1:8188/prompt";

        let client = reqwest::blocking::Client::new();

        let resp = client.post(url).json(compute_graph).send().unwrap();

        let json = resp.json::<serde_json::Value>();

        println!("{:?}", json);
    }

    pub fn fetch_image(image_url: String) -> image::RgbImage {
        let resp = reqwest::blocking::get(image_url).unwrap();

        let img = image::load_from_memory(&resp.bytes().unwrap()).unwrap();

        img.into_rgb8()
    }
}
