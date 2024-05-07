use self::{command_palette::CommandPalette, menu::Menu, node_view::NodeView, tabs::Tabs};
use crate::{
    model::{Graph, Link, LinkType, Model, Node, NodeType},
    ui::View,
    utils::{Aro, Arw},
};
use slint::Weak;
use std::{
    collections::HashMap,
    fs::File,
    sync::mpsc::{Receiver, Sender},
};

mod command_palette;
mod menu;
mod node_view;
mod tabs;

#[derive(Debug)]
pub enum Event {
    SetNodePosition(usize, f32, f32),
    AddNode(NodeType),
    AddLink(Link),
    RemoveLink(usize),
    SelectTab(usize),
    CloseTab(usize),
    NewTab,
    SetCommandSearch(String),
    Save,
    SaveAs,
    OpenFile,
}

trait Controller {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>);
    fn notify(ui: &View, model: &Model, evt: &Event);
}

pub struct Mediator {
    rx: Receiver<Event>,
    model: Arw<Model>,
}

impl Mediator {
    pub fn new(ui: &View, mut model: Model) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        populate_available_nodes(&mut model);

        let model = Arw::new(model);
        let ro_model = Aro::from(model.clone());

        Menu::setup(ro_model.clone(), ui, tx.clone());
        Tabs::setup(ro_model.clone(), ui, tx.clone());
        NodeView::setup(ro_model.clone(), ui, tx.clone());
        CommandPalette::setup(ro_model.clone(), ui, tx.clone());

        Self { rx, model }
    }

    pub fn run(self, ui: Weak<View>) {
        for evt in self.rx.iter() {
            macro_rules! notify {
                ($($ctrl:ty),*) => {
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read();
                            $(<$ctrl>::notify(&ui, &model, &evt);)*
                        }
                    })
                    .unwrap()
                };
            }
            println!("{:?}", &evt);

            use Event::*;
            match evt {
                SetCommandSearch(ref query) => {
                    let mut model = self.model.write();
                    model.set_command_search(query.clone());

                    notify!(CommandPalette);
                }
                SetNodePosition(node_idx, x, y) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().set_node_position(node_idx, x, y);
                    }
                    notify!(NodeView);
                }
                AddNode(ref ty) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().add_node(ty.clone());
                    }
                    notify!(NodeView);
                }
                AddLink(ref lnk) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().add_link(lnk.clone());
                    }
                    notify!(NodeView);
                }
                RemoveLink(i) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().remove_link(i);
                    }
                    notify!(NodeView);
                }
                SelectTab(i) => {
                    let mut model = self.model.write();
                    model.tabs_mut().select_tab(i);
                    notify!(NodeView, Tabs, CommandPalette);
                }
                NewTab => {
                    let mut model = self.model.write();
                    model.tabs_mut().new_tab();
                    populate_available_nodes(&mut model);
                    notify!(NodeView, Tabs, CommandPalette);
                }
                CloseTab(i) => {
                    let mut model = self.model.write();
                    model.tabs_mut().close_tab(i);
                    notify!(NodeView, Tabs, CommandPalette);
                }
                Save => {
                    let mut model = self.model.write();
                    if let Some(selected) = model.tabs_mut().selected_project_mut() {
                        if let Some(path) = selected.file_path() {
                            save_graph(path, selected.graph());
                        } else if let Some(path) = save_dialog() {
                            selected.set_file_path(path.clone());
                            save_graph(&path, selected.graph());
                        }
                    }
                    notify!(Tabs);
                }
                SaveAs => {
                    let mut model = self.model.write();
                    if let Some(selected) = model.tabs_mut().selected_project_mut() {
                        if let Some(path) = save_dialog() {
                            selected.set_file_path(path.clone());
                            save_graph(&path, selected.graph());
                        }
                    }
                    notify!(Tabs);
                }
                OpenFile => {
                    let mut model = self.model.write();
                    if let Some(path) = open_dialog() {
                        // TODO: refactor project initialization into model
                        let graph = read_graph(&path);
                        model.tabs_mut().new_tab();
                        let selected = model.tabs_mut().selected_project_mut().unwrap();
                        *selected.graph_mut() = graph;
                        selected.set_file_path(path.clone());
                        populate_available_nodes(&mut model);
                    }
                    notify!(NodeView, Tabs, CommandPalette);
                }
            }
        }
    }
}

fn open_dialog() -> Option<String> {
    // TODO: better error handling
    native_dialog::FileDialog::new()
        .add_filter("Kira Graph File", &["kira"])
        .show_open_single_file()
        .ok()
        .flatten()
        .and_then(|pb| pb.to_str().map(|s| s.to_owned()))
}

fn save_dialog() -> Option<String> {
    // TODO: better error handling
    native_dialog::FileDialog::new()
        .add_filter("Kira Graph File", &["kira"])
        .show_save_single_file()
        .ok()
        .flatten()
        .and_then(|pb| pb.to_str().map(|s| s.to_owned()))
}

fn save_graph(path: &str, graph: &Graph) {
    // TODO: better error handling
    let f = File::create(path).unwrap();
    serde_json::to_writer(f, graph).unwrap();
}

fn read_graph(path: &str) -> Graph {
    // TODO: better error handling
    let f = File::open(path).unwrap();
    serde_json::from_reader(f).unwrap()
}

fn populate_available_nodes(model: &mut Model) {
    let mut dummy_nodes: HashMap<NodeType, Node> = HashMap::new();
    for i in 0..5 {
        let name = format!("A{}", i);
        dummy_nodes.insert(
            NodeType(name.clone()),
            Node {
                inputs: vec![
                    ("Text".into(), "TXT".into()),
                    ("Image".into(), "IMG".into()),
                ],
                outputs: vec![
                    ("Text".into(), "TXT".into()),
                    ("Image".into(), "IMG".into()),
                ],
                name,
                description: "Node of type A".into(),
                category: "Dummy".into(),
            },
        );
        let name = format!("B{}", i);
        dummy_nodes.insert(
            NodeType(name.clone()),
            Node {
                inputs: vec![("Image".into(), "IMG".into())],
                outputs: vec![],
                name,
                description: "Node of type B".into(),
                category: "Dummy".into(),
            },
        );
        let name = format!("C{}", i);
        dummy_nodes.insert(
            NodeType(name.clone()),
            Node {
                inputs: vec![],
                outputs: vec![("Text".into(), "TXT".into()), ("Text".into(), "TXT".into())],
                name,
                description: "Node of type C".into(),
                category: "Dummy".into(),
            },
        );
    }

    let available_nodes = model
        .backend()
        .query_available_nodes()
        .map(|hm| {
            hm.into_iter()
                .map(|(k, v)| {
                    (
                        NodeType(k),
                        Node {
                            inputs: v
                                .input
                                .into_iter()
                                .map(|(lbl, ty)| (lbl, LinkType(ty)))
                                .collect(),
                            outputs: v
                                .output_name
                                .into_iter()
                                .zip(v.output)
                                .map(|(lbl, ty)| (lbl, LinkType(ty)))
                                .collect(),
                            name: v.display_name,
                            description: v.description,
                            category: v.category,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or(dummy_nodes);
    if let Some(project) = model.tabs_mut().selected_project_mut() {
        project.set_available_nodes(available_nodes);
    }
}
