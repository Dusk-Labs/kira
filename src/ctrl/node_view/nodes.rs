use super::{Aro, Controller};
use crate::{
    ctrl::Event,
    model::Model,
    ui::{self, NodeData, Slot, View},
};
use slint::{ComponentHandle, VecModel};
use std::sync::mpsc::Sender;

pub struct Nodes;

impl Controller for Nodes {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        ui.global::<ui::NodeLogic>().on_move_node({
            move |node_idx, x, y| {
                tx.send(Event::SetNodePosition(node_idx as usize, x, y))
                    .unwrap();
            }
        });
        let model = model.read();
        refresh(ui, &model);
    }

    fn notify(ui: &View, model: &Model, _evt: &Event) {
        refresh(ui, model);
    }
}

fn refresh(ui: &View, model: &Model) {
    let project = model.tabs().selected_project();
    ui.set_nodes(VecModel::from_slice(
        &project
            .get_nodes()
            .iter()
            .map(|ni| {
                let n = project.get_available_node(&ni.ty).unwrap();
                NodeData {
                    inputs: VecModel::from_slice(
                        &n.inputs
                            .iter()
                            .map(|(name, ty)| Slot {
                                name: name.clone().into(),
                                ty: ty.clone().into(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    outputs: VecModel::from_slice(
                        &n.outputs
                            .iter()
                            .map(|(name, ty)| Slot {
                                name: name.clone().into(),
                                ty: ty.clone().into(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    text: n.name.clone().into(),
                    width: 100.,
                    x: ni.pos.0,
                    y: ni.pos.1,
                }
            })
            .collect::<Vec<_>>(),
    ))
}
