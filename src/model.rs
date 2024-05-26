pub use self::{
    backend::Backend,
    backend::node::{TY_FLOAT, TY_INT, TY_SELECT, TY_STRING},
    backend::conv::WorkflowBuilder,
    backend::workflow::WorkflowPrompt,
    tabs::{
        project::{
            graph::Graph,
            graph::{Link, LinkType, NodeType},
            Node,
        },
        Tabs,
    },
};

mod backend;
mod file;
mod tabs;

use crate::ctrl::Event;

#[derive(Debug)]
pub struct Model {
    tabs: Tabs,
    backend: Backend,
    command_search: String,
}

impl Model {
    pub fn new() -> Self {
        Self {
            tabs: Tabs::new(),
            backend: Backend::new(),
            command_search: "".into(),
        }
    }
    pub fn spawn_client(&self, tx: std::sync::mpsc::Sender<Event>) {
        self.backend.spawm_client(tx);
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn tabs(&self) -> &Tabs {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut Tabs {
        &mut self.tabs
    }

    pub fn command_search(&self) -> &String {
        &self.command_search
    }

    pub fn set_command_search(&mut self, query: String) {
        self.command_search = query;
    }
}
