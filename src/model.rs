pub use self::{
    backend::Backend,
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
