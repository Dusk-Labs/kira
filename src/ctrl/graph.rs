use self::{floating::Floating, links::Links, nodes::Nodes};
use super::{Aro, Controller};
use crate::{
    ctrl::Event,
    model::Model,
    ui::{GraphLogic, View},
};
use slint::ComponentHandle;
use std::sync::mpsc::Sender;

mod floating;
mod links;
mod nodes;

pub struct Graph;

impl Controller for Graph {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        Nodes::setup(model.clone(), ui, tx.clone());
        Links::setup(model.clone(), ui, tx.clone());
        Floating::setup(model.clone(), ui, tx.clone());

        let model = model.read();
        refresh(ui, &model);

        ui.global::<GraphLogic>().on_set_zoom({
            let tx = tx.clone();
            move |zoom| {
                tx.send(Event::SetZoom(zoom)).unwrap();
            }
        });
        ui.global::<GraphLogic>().on_set_offset({
            let tx = tx.clone();
            move |x, y| {
                tx.send(Event::SetOffset(x, y)).unwrap();
            }
        });
    }

    fn notify(ui: &View, model: &Model, evt: &Event) {
        refresh(ui, model);

        use Event::*;
        match evt {
            SetOffset(..) | SetZoom(..) | OpenFile | SetNodePosition(..) | CloseTab(..)
            | SelectTab(..) | NewTab => {
                Nodes::notify(ui, model, evt);
                Links::notify(ui, model, evt);
            }
            RemoveLink(..) | AddLink(..) => {
                Links::notify(ui, model, evt);
            }
            AddNode(..) => {
                Nodes::notify(ui, model, evt);
            }
            Save | SaveAs | SetCommandSearch(..) => {}
        }
    }
}

fn refresh(ui: &View, model: &Model) {
    if let Some(project) = model.tabs().selected_project() {
        let graph = project.graph();
        ui.set_zoom(graph.zoom());
        let (x, y) = graph.offset();
        ui.set_offset_x(x);
        ui.set_offset_y(y);
    }
}
