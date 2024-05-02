use std::cell::RefCell;
use std::rc::Rc;

pub use self::backend::Backend;
pub use self::project::Project;

mod backend;
mod file;
pub mod project;

pub struct Model {
    project: Rc<RefCell<Project>>,
    backend: Backend,
}

impl Model {
    pub fn new() -> Self {
        Self {
            project: Rc::new(RefCell::new(Project::new())),
            backend: Backend::new(),
        }
    }
    pub fn backend(&self) -> &Backend {
        &self.backend
    }
    pub fn project(&self) -> Rc<RefCell<Project>> {
        self.project.clone()
    }
}
