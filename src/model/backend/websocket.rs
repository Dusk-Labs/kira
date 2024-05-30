use async_trait::async_trait;
use ezsockets::client::ClientExt;
use ezsockets::ClientConfig;
use serde::Deserialize;
use uuid::Uuid;

use crate::ctrl::Event;
use std::sync::mpsc::Sender;

pub struct WsClient {
    tx: Sender<Event>,
    base_url: String,
    client_id: Uuid,
}

impl WsClient {
    pub fn new(tx: Sender<Event>, base_url: String, client_id: Uuid) -> Self {
        Self {
            tx,
            base_url,
            client_id,
        }
    }

    pub fn client_id(&self) -> String {
        let mut buffer = Uuid::encode_buffer();
        let uuid = self.client_id.as_simple().encode_lower(&mut buffer);

        uuid.to_string()
    }

    pub async fn listen(self) {
        let url = format!("{}/ws?clientId={}", self.base_url, self.client_id());
        let config = ClientConfig::new(url.as_str());

        let (handle, future) = ezsockets::connect(move |_client| self, config).await;

        tokio::spawn(async move {
            future.await.unwrap();
        });

        std::mem::forget(handle);
    }
}

#[async_trait]
impl ClientExt for WsClient {
    type Call = ();

    async fn on_call(&mut self, _: Self::Call) -> Result<(), ezsockets::Error> {
        Ok(())
    }

    async fn on_binary(&mut self, _: Vec<u8>) -> Result<(), ezsockets::Error> {
        Ok(())
    }

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        let mut de = serde_json::Deserializer::from_str(&text);
        let data: Message = serde_path_to_error::deserialize(&mut de).unwrap();

        println!("{:?}", data);

        if let Message::Executed { node, output, .. } = data {
            let node: usize = node.parse().unwrap();
            let first = &output.images[0];
            let output = format!(
                "{}/view?filename={}&type=temp",
                self.base_url, first.filename
            );

            self.tx.send(Event::SetNodeOutput { node, output }).unwrap();
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Status {
        status: Status,
    },
    ExecutionStart {
        prompt_id: Option<String>,
    },
    ExecutionCached {
        prompt_id: String,
        nodes: Vec<String>,
    },
    Executing {
        node: Option<String>,
        prompt_id: Option<String>,
    },
    Progress {
        node: String,
        value: usize,
        max: usize,
        prompt_id: String,
    },
    Executed {
        node: String,
        output: NodeOutput,
        prompt_id: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub exec_info: ExecInfo,
}

#[derive(Deserialize, Debug)]
pub struct ExecInfo {
    pub queue_remaining: usize,
}

#[derive(Deserialize, Debug)]
pub struct NodeOutput {
    pub images: Vec<ImageOutput>,
}

#[derive(Deserialize, Debug)]
pub struct ImageOutput {
    pub filename: String,
    pub subfolder: String,
    #[serde(rename = "type")]
    pub ty: String,
}
