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

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Backend {
    rt: runtime::Runtime,
    base_url: String,
    client: reqwest::blocking::Client,
    client_id: uuid::Uuid,
}

impl Backend {
    pub fn new() -> Self {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let client = reqwest::blocking::ClientBuilder::new()
            .build()
            .expect("Failed to build backend client.");

        Self {
            rt,
            base_url: "http://127.0.0.1:8188".into(),
            client,
            client_id: uuid::Uuid::now_v7(),
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn client_id(&self) -> String {
        let mut buffer = uuid::Uuid::encode_buffer();
        let uuid = self.client_id.as_simple().encode_lower(&mut buffer);

        uuid.to_string()
    }

    pub fn spawm_client(&self, tx: std::sync::mpsc::Sender<Event>) {
        self.rt
            .block_on(websocket::WsClient::new(tx, self.base_url.clone(), self.client_id).listen());
    }

    pub fn query_available_nodes(&self) -> Result<HashMap<String, Node>> {
        let url = format!("{}/object_info", self.base_url);

        let response = self.client.get(url).send()?;

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

    pub fn compute_graph(&self, compute_graph: &workflow::WorkflowPrompt) -> Result<()> {
        let url = format!("{}/prompt", self.base_url);

        let resp = self.client.post(url).json(compute_graph).send()?;

        let json = resp.json::<serde_json::Value>();

        println!("{:?}", json);

        Ok(())
    }

    pub fn fetch_image(&self, image_url: String) -> Result<image::RgbImage> {
        let resp = self.client.get(image_url).send()?;
        let img = image::load_from_memory(&resp.bytes()?)?;

        Ok(img.into_rgb8())
    }
}
