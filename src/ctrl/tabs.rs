use super::{Aro, Controller, Event};
use crate::{
    model::Model,
    ui::{TabLogic, View},
};
use slint::{ComponentHandle, SharedString, VecModel};
use std::sync::mpsc::Sender;

pub struct Tabs;

impl Controller for Tabs {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        let model = model.read();
        refresh(&model, ui);

        ui.global::<TabLogic>().on_select_tab({
            let tx = tx.clone();
            move |selected| {
                tx.send(Event::SelectTab(selected as usize)).unwrap();
            }
        });
        ui.global::<TabLogic>().on_new_tab({
            let tx = tx.clone();
            move || {
                tx.send(Event::NewTab).unwrap();
            }
        });
    }
    fn notify(ui: &View, model: &Model, evt: &Event) {
        use Event::*;
        match evt {
            SelectTab(..) | NewTab => {
                refresh(model, ui);
            }
            SetCommandSearch(..) | SetNodePosition(..) | AddNode(..) | RemoveLink(..)
            | AddLink(..) => {}
        }
    }
}

fn refresh(model: &Model, ui: &View) {
    let mut dummy_names = vec![];
    for i in 0..model.tabs().len() {
        dummy_names.push(SharedString::from(format!("Tab {i}")));
    }

    ui.set_tab_names(VecModel::from_slice(&dummy_names));
    ui.set_selected_tab(model.tabs().selected_tab() as i32);
}
