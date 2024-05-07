use project::Project;

pub mod project;

#[derive(Debug)]
pub struct Tabs {
    tabs: Vec<Project>,
    selected_tab: Option<usize>,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            tabs: vec![Project::new()],
            selected_tab: Some(0),
        }
    }
    pub fn selected_project(&self) -> Option<&Project> {
        self.selected_tab.map(|i| &self.tabs[i])
    }
    pub fn selected_project_mut(&mut self) -> Option<&mut Project> {
        self.selected_tab.map(|i| &mut self.tabs[i])
    }
    pub fn select_tab(&mut self, tab: usize) {
        self.selected_tab = Some(tab);
    }
    pub fn selected_tab(&self) -> Option<usize> {
        self.selected_tab
    }
    pub fn new_tab(&mut self) {
        self.tabs.push(Project::new());
        self.selected_tab = Some(self.tabs.len() - 1);
    }
    pub fn close_tab(&mut self, tab: usize) {
        self.tabs.remove(tab);
        self.selected_tab = if self.tabs.is_empty() {
            None
        } else if let Some(selected) = self.selected_tab {
            Some(selected.min(self.tabs.len() - 1))
        } else {
            None
        }
    }
    pub fn tab_titles(&self) -> Vec<String> {
        self.tabs
            .iter()
            .map(|t| t.file_path().unwrap_or("Untitled").into())
            .collect()
    }
}
