use std::sync::{Arc, RwLock};

pub use self::{backend::Backend, project::Project};

mod backend;
mod file;
pub mod project;

#[derive(Debug)]
pub struct Model {
    project: Arc<RwLock<Project>>,
    backend: Backend,
}

impl Model {
    pub fn new() -> Self {
        Self {
            project: Arc::new(RwLock::new(Project::new())),
            backend: Backend::new(),
        }
    }
    pub fn backend(&self) -> &Backend {
        &self.backend
    }
    pub fn project(&self) -> Arc<RwLock<Project>> {
        self.project.clone()
    }
}
