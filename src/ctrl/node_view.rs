use self::{floating::Floating, links::Links, nodes::Nodes};
use super::{Aro, Controller};
use crate::{ctrl::Event, model::Model, ui::View};
use std::sync::mpsc::Sender;

mod floating;
mod links;
mod nodes;

pub struct NodeView;

impl Controller for NodeView {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        Nodes::setup(model.clone(), ui, tx.clone());
        Links::setup(model.clone(), ui, tx.clone());
        Floating::setup(model.clone(), ui, tx.clone());

        ui.set_zoom(2.);
    }

    fn notify(ui: &View, model: &Model, evt: &Event) {
        use Event::*;
        match evt {
            OpenFile | SetNodePosition(..) | CloseTab(..) | SelectTab(..) | NewTab => {
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
