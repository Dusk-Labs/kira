use crate::{
    model::{
        self,
        project::{self, Node, NodeType},
        Model,
    },
    ui::View,
};
use slint::Weak;
use std::{collections::HashMap, sync::mpsc::Receiver};

mod command_palette;
mod node_view;

pub enum Event {
    SetNodePosition(usize, f32, f32),
    AddLink(model::project::Link),
    RemoveLink(usize),
    AddNode(model::project::NodeType),
}

pub struct Controller {
    rx: Receiver<Event>,
    model: Model,
}

impl Controller {
    pub fn new(ui: &View, mut model: Model) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        populate_available_nodes(&mut model);
        node_view::setup(model.project(), ui, tx.clone());
        command_palette::setup(model.project(), ui, tx);
        Self { rx, model }
    }

    pub fn run(self, ui: Weak<View>) {
        for evt in self.rx.iter() {
            use Event::*;
            match evt {
                SetNodePosition(node_idx, x, y) => {
                    self.model.project().write().unwrap().set_node_position(
                        node_idx as usize,
                        x,
                        y,
                    );
                    node_view::notify(ui.clone(), self.model.project(), evt);
                }

                AddLink(ref lnk) => {
                    self.model.project().write().unwrap().add_link(lnk.clone());
                    node_view::notify(ui.clone(), self.model.project(), evt);
                }

                RemoveLink(i) => {
                    self.model.project().write().unwrap().remove_link(i);
                    node_view::notify(ui.clone(), self.model.project(), evt);
                }

                AddNode(ref ty) => {
                    self.model.project().write().unwrap().add_node(ty.clone());
                    node_view::notify(ui.clone(), self.model.project(), evt);
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
        .project()
        .write()
        .unwrap()
        .set_available_nodes(available_nodes);
}
