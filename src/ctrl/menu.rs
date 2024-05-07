use super::{Controller, Event};
use crate::{
    model::Model,
    ui::{MenuLogic, View},
    utils::Aro,
};
use slint::ComponentHandle;
use std::sync::mpsc::Sender;

pub struct Menu;

impl Controller for Menu {
    fn setup(_model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        // let model = model.read();
        ui.global::<MenuLogic>().on_open_file({
            let tx = tx.clone();
            move || tx.send(Event::OpenFile).unwrap()
        });
        ui.global::<MenuLogic>().on_save({
            let tx = tx.clone();
            move || tx.send(Event::Save).unwrap()
        });
        ui.global::<MenuLogic>().on_save_as({
            let tx = tx.clone();
            move || tx.send(Event::SaveAs).unwrap()
        });
    }
    fn notify(_ui: &View, _model: &Model, _evt: &Event) {}
}
