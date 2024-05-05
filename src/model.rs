pub use self::{backend::Backend, project::Project};

mod backend;
mod file;
pub mod project;

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

#[derive(Debug)]
pub struct Tabs {
    tabs: Vec<Project>,
    selected_tab: usize,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            tabs: vec![Project::new()],
            selected_tab: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.tabs.len()
    }
    pub fn selected_project(&self) -> &Project {
        &self.tabs[self.selected_tab]
    }
    pub fn selected_project_mut(&mut self) -> &mut Project {
        &mut self.tabs[self.selected_tab]
    }
    pub fn select_tab(&mut self, tab: usize) {
        self.selected_tab = tab;
    }
    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }
    pub fn new_tab(&mut self) {
        self.tabs.push(Project::new());
        self.selected_tab = self.tabs.len() - 1;
    }
}
