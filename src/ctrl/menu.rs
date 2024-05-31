use super::Controller;
use super::Event;

use crate::model::Model;
use crate::ui::MenuLogic;
use crate::ui::View;
use crate::utils::Aro;

use slint::ComponentHandle;
use std::sync::mpsc::Sender;

pub mod darwin;

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
        ui.global::<MenuLogic>().on_render({
            let tx = tx.clone();
            move || tx.send(Event::Render).unwrap()
        });
    }
    fn notify(_ui: &View, _model: &Model, _evt: &Event) {}
}
