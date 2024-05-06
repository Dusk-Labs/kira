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
        ui.global::<TabLogic>().on_close_tab({
            let tx = tx.clone();
            move |closing| {
                tx.send(Event::CloseTab(closing as usize)).unwrap();
            }
        });
    }
    fn notify(ui: &View, model: &Model, evt: &Event) {
        use Event::*;
        match evt {
            SelectTab(..) | NewTab | CloseTab(..) => {
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
    ui.set_selected_tab({
        let selected = model.tabs().selected_tab();
        selected
            .map(i32::try_from)
            .transpose()
            .unwrap()
            .unwrap_or(-1)
    });
}
