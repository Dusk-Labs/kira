use crate::{
    model::{
        project::{self, Node, NodeType},
        Model,
    },
    ui::View,
};
use std::{collections::HashMap, rc::Rc};

mod command_palette;
mod node_view;

pub fn setup(ui: Rc<View>, model: &mut Model) {
    populate_available_nodes(model);
    node_view::setup(ui.clone(), model.project());
    command_palette::setup(ui, model.project());
}

fn populate_available_nodes(model: &mut Model) {
    let dummy_nodes: HashMap<NodeType, Node> = [
        (
            "A".into(),
            Node {
                inputs: vec![],
                outputs: vec![
                    ("Text".into(), "TXT".into()),
                    ("Image".into(), "IMG".into()),
                ],
                name: "A".into(),
                description: "Node of type A".into(),
                category: "Dummy".into(),
            },
        ),
        (
            "b".into(),
            Node {
                inputs: vec![
                    ("Text".into(), "TXT".into()),
                    ("Image".into(), "IMG".into()),
                ],
                outputs: vec![],
                name: "B".into(),
                description: "Node of type B".into(),
                category: "Dummy".into(),
            },
        ),
    ]
    .into();
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
        .borrow_mut()
        .set_available_nodes(available_nodes);
}
