use self::{command_palette::CommandPalette, graph::Graph, menu::Menu, tabs::Tabs};
use crate::{
    model::{self, Link, LinkType, Model, Node, NodeType, WorkflowPrompt},
    ui::View,
    utils::{Aro, Arw},
};
use slint::{ComponentHandle, Weak};
use std::{
    collections::HashMap,
    fs::File,
    sync::mpsc::{Receiver, Sender},
};

mod command_palette;
mod graph;
mod menu;
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
    SetZoom(f32),
    SetOffset(f32, f32),
    Save,
    SaveAs,
    OpenFile,
    SetField {
        node_idx: usize,
        input: String,
        ty: String,
        value: String,
    },
    Render,
    SetNodeOutput {
        node: usize,
        output: String,
    },
}

trait Controller {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>);
    fn notify(ui: &View, model: &Model, evt: &Event);
}

pub struct Mediator {
    rx: Receiver<Event>,
    model: Arw<Model>,
    ui: Weak<View>,
}

impl Mediator {
    pub fn new(ui: &View, mut model: Model) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        populate_available_nodes(&mut model);

        model.spawn_client(tx.clone());

        let model = Arw::new(model);
        let ro_model = Aro::from(model.clone());

        Menu::setup(ro_model.clone(), ui, tx.clone());
        Tabs::setup(ro_model.clone(), ui, tx.clone());
        Graph::setup(ro_model.clone(), ui, tx.clone());
        CommandPalette::setup(ro_model.clone(), ui, tx.clone());

        Self {
            rx,
            model,
            ui: ui.as_weak(),
        }
    }

    pub fn run(self) {
        for evt in self.rx.iter() {
            macro_rules! notify {
                ($($ctrl:ty),*) => {
                    self.ui.upgrade_in_event_loop({
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
                    notify!(Graph);
                }
                AddNode(ref ty) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        let n = project.get_available_node(ty).unwrap();
                        let fields = n.fields.into_iter().collect::<Vec<_>>();

                        project.graph_mut().add_node(ty.clone(), fields);
                    }
                    notify!(Graph);
                }
                AddLink(ref lnk) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().add_link(lnk.clone());
                    }
                    notify!(Graph);
                }
                RemoveLink(i) => {
                    let mut model = self.model.write();
                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        project.graph_mut().remove_link(i);
                    }
                    notify!(Graph);
                }
                SelectTab(i) => {
                    let mut model = self.model.write();
                    model.tabs_mut().select_tab(i);
                    notify!(Graph, Tabs, CommandPalette);
                }
                NewTab => {
                    let mut model = self.model.write();
                    model.tabs_mut().new_tab();
                    populate_available_nodes(&mut model);
                    notify!(Graph, Tabs, CommandPalette);
                }
                CloseTab(i) => {
                    let mut model = self.model.write();
                    model.tabs_mut().close_tab(i);
                    notify!(Graph, Tabs, CommandPalette);
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
                        dbg!(&graph);
                        *selected.graph_mut() = graph;
                        selected.set_file_path(path.clone());
                        populate_available_nodes(&mut model);
                    }
                    notify!(Graph, Tabs, CommandPalette);
                }
                SetZoom(zoom) => {
                    let mut model = self.model.write();
                    if let Some(selected) = model.tabs_mut().selected_project_mut() {
                        selected.graph_mut().set_zoom(zoom);
                    }
                    notify!(Graph);
                }
                SetOffset(x, y) => {
                    let mut model = self.model.write();
                    if let Some(selected) = model.tabs_mut().selected_project_mut() {
                        selected.graph_mut().set_offset((x, y));
                    }
                    notify!(Graph);
                }
                SetField {
                    node_idx,
                    ref input,
                    ref ty,
                    ref value,
                } => {
                    use crate::model::{TY_FLOAT, TY_INT, TY_SELECT, TY_STRING};

                    let mut model = self.model.write();
                    let Some(project) = model.tabs_mut().selected_project_mut() else {
                        continue;
                    };

                    let Some(state) = project.graph_mut().get_state_mut(node_idx, input) else {
                        continue;
                    };

                    match ty.as_str() {
                        TY_STRING | TY_SELECT => state.set_text(value.clone()),
                        TY_INT => {
                            let _ = value.parse().map(|x| state.set_int(x));
                        }
                        TY_FLOAT => {
                            let _ = value.parse().map(|x| state.set_float(x));
                        }
                        _ => {}
                    }
                }
                Render => {
                    let mut model = self.model.write();
                    let Some(project) = model.tabs_mut().selected_project_mut() else {
                        continue;
                    };

                    let available_nodes = project.available_nodes();
                    let wf = WorkflowPrompt::from_graph(available_nodes, project.graph())
                        .with_client_id(model.backend().client_id());

                    println!("\n");
                    println!("{:#?}", &wf);

                    let _ = model.backend().compute_graph(&wf);
                }
                SetNodeOutput { node, ref output } => {
                    println!("Output for node idx {} is at path {}", node, output);

                    let mut model = self.model.write();
                    let image = model.backend().fetch_image(output.into());

                    if let Some(project) = model.tabs_mut().selected_project_mut() {
                        let node = project.graph_mut().get_node_mut(node).unwrap();
                        node.image = image.ok();
                        notify!(Graph);
                    };

                }
            }
        }
    }
}

fn open_dialog() -> Option<String> {
    // TODO: better error handling
    let (tx, rx) = std::sync::mpsc::channel();
    slint::invoke_from_event_loop(move || {
        tx.send(
            native_dialog::FileDialog::new()
                .add_filter("Kira Graph File", &["kira"])
                .show_open_single_file()
                .ok()
                .flatten()
                .and_then(|pb| pb.to_str().map(|s| s.to_owned())),
        )
        .unwrap()
    })
    .unwrap();
    rx.recv().unwrap()
}

fn save_dialog() -> Option<String> {
    // TODO: better error handling
    let (tx, rx) = std::sync::mpsc::channel();
    slint::invoke_from_event_loop(move || {
        tx.send(
            native_dialog::FileDialog::new()
                .add_filter("Kira Graph File", &["kira"])
                .show_save_single_file()
                .ok()
                .flatten()
                .and_then(|pb| pb.to_str().map(|s| s.to_owned())),
        )
        .unwrap()
    })
    .unwrap();
    rx.recv().unwrap()
}

fn save_graph(path: &str, graph: &model::Graph) {
    // TODO: better error handling
    let f = File::create(path).unwrap();
    serde_json::to_writer(f, graph).unwrap();
}

fn read_graph(path: &str) -> model::Graph {
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
                fields: vec![],
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
                fields: vec![],
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
                fields: vec![],
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
                                .iter()
                                .filter(|(_, ty)| ty.is_connection())
                                .map(|(lbl, ty)| (lbl.clone(), LinkType(ty.ty())))
                                .collect(),
                            outputs: v
                                .output_name
                                .into_iter()
                                .zip(v.output)
                                .map(|(lbl, ty)| (lbl, LinkType(ty)))
                                .collect(),
                            fields: v
                                .input
                                .iter()
                                .filter(|(_, ty)| !ty.is_connection())
                                .map(|(lbl, ty)| (lbl.clone(), ty.clone()))
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
