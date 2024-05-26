use async_trait::async_trait;
use ezsockets::client::ClientExt;
use ezsockets::ClientConfig;
use serde::Deserialize;

use std::sync::mpsc::Sender;
use crate::ctrl::Event;

pub struct WsClient {
    tx: Sender<Event>
}

impl WsClient {
    pub fn new(tx: Sender<Event>) -> Self {
        Self {
            tx
        }
    }

    pub async fn listen(mut self) -> () {
        let config = ClientConfig::new("ws://127.0.0.1:8188/ws?clientId=f9e9494bb05849738d26b3b914e3eec2");

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

        match data {
            Message::Executed { node, output, .. } => {
                let node: usize = node.parse().unwrap();
                let first = &output.images[0];
                let output = format!("http://127.0.0.1:8188/view?filename={}&type=temp", first.filename);

                self.tx.send(Event::SetNodeOutput { node, output }).unwrap();
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Status {
        status: Status
    },
    ExecutionStart {
        prompt_id: Option<String>
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
    }
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub exec_info: ExecInfo,
}

#[derive(Deserialize, Debug)]
pub struct ExecInfo {
    pub queue_remaining: usize
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
    pub ty: String
}
