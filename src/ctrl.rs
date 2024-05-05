use crate::{
    model::{
        self,
        project::{self, Node, NodeType},
        Model,
    },
    ui::View,
};
use slint::Weak;
use std::{
    collections::HashMap,
    sync::{mpsc::Receiver, Arc, RwLock},
};

mod command_palette;
mod node_view;
mod tabs;

#[derive(Debug)]
pub enum Event {
    SetNodePosition(usize, f32, f32),
    AddNode(model::project::NodeType),
    AddLink(model::project::Link),
    RemoveLink(usize),
    SelectTab(usize),
    NewTab,
    SetCommandSearch(String),
}

pub struct Mediator {
    rx: Receiver<Event>,
    model: Arc<RwLock<Model>>,
}

impl Mediator {
    pub fn new(ui: &View, mut model: Model) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        populate_available_nodes(&mut model);

        let model = Arc::new(RwLock::new(model));

        tabs::setup(model.clone(), ui, tx.clone());
        node_view::setup(model.clone(), ui, tx.clone());
        command_palette::setup(model.clone(), ui, tx.clone());

        Self { rx, model }
    }

    pub fn run(self, ui: Weak<View>) {
        for evt in self.rx.iter() {
            dbg!(&evt);
            use Event::*;
            match evt {
                SetCommandSearch(ref query) => {
                    let mut model = self.model.write().unwrap();
                    model.set_command_search(query.clone());

                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            command_palette::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                SetNodePosition(node_idx, x, y) => {
                    let mut model = self.model.write().unwrap();
                    model
                        .tabs_mut()
                        .selected_project_mut()
                        .set_node_position(node_idx, x, y);
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                AddNode(ref ty) => {
                    let mut model = self.model.write().unwrap();
                    model.tabs_mut().selected_project_mut().add_node(ty.clone());
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                AddLink(ref lnk) => {
                    let mut model = self.model.write().unwrap();
                    model
                        .tabs_mut()
                        .selected_project_mut()
                        .add_link(lnk.clone());
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                RemoveLink(i) => {
                    let mut model = self.model.write().unwrap();
                    model.tabs_mut().selected_project_mut().remove_link(i);
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                SelectTab(i) => {
                    let mut model = self.model.write().unwrap();
                    model.tabs_mut().select_tab(i);
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                            tabs::notify(&ui, &model, &evt);
                            command_palette::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
                NewTab => {
                    let mut model = self.model.write().unwrap();
                    model.tabs_mut().new_tab();
                    ui.upgrade_in_event_loop({
                        let model = self.model.clone();
                        move |ui| {
                            let model = model.read().unwrap();
                            node_view::notify(&ui, &model, &evt);
                            tabs::notify(&ui, &model, &evt);
                            command_palette::notify(&ui, &model, &evt);
                        }
                    })
                    .unwrap()
                }
            }
        }
    }
}

fn populate_available_nodes(model: &mut Model) {
    let mut dummy_nodes: HashMap<NodeType, Node> = HashMap::new();
    for i in 0..20 {
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
    }

    let available_nodes = model
        .backend()
        .query_available_nodes()
        .map(|hm| {
            hm.into_iter()
                .map(|(k, v)| {
                    (
                        project::NodeType(k),
                        project::Node {
                            inputs: v
                                .input
                                .into_iter()
                                .map(|(lbl, ty)| (lbl, project::LinkType(ty)))
                                .collect(),
                            outputs: v
                                .output_name
                                .into_iter()
                                .zip(v.output)
                                .map(|(lbl, ty)| (lbl, project::LinkType(ty)))
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
    model
        .tabs_mut()
        .selected_project_mut()
        .set_available_nodes(available_nodes);
}
