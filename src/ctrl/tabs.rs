use super::{Aro, Controller, Event};
use crate::{
    model::Model,
    ui::{TabLogic, View},
};
use slint::{ComponentHandle, SharedString, VecModel};
use std::{path::Path, sync::mpsc::Sender};

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
    fn notify(ui: &View, model: &Model, _evt: &Event) {
        refresh(model, ui);
    }
}

fn refresh(model: &Model, ui: &View) {
    let tab_titles = model
        .tabs()
        .tab_titles()
        .iter()
        .map(|path| {
            Path::new(path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .into()
        })
        .collect::<Vec<SharedString>>();

    ui.set_tab_names(VecModel::from_slice(&tab_titles));
    ui.set_selected_tab({
        let selected = model.tabs().selected_tab();
        selected
            .map(i32::try_from)
            .transpose()
            .unwrap()
            .unwrap_or(-1)
    });
}
