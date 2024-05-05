use super::Event;
use crate::{
    model::Model,
    ui::{PaletteSearch, SearchItem, View},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::sync::{mpsc::Sender, Arc, RwLock};

pub fn setup(model: Arc<RwLock<Model>>, ui: &View, tx: Sender<Event>) {
    let model = model.read().unwrap();
    setup_command_palette_logic(ui, tx);
    refresh(&model, ui);
}

fn setup_command_palette_logic(ui: &View, tx: Sender<Event>) {
    ui.global::<PaletteSearch>().on_search({
        let tx = tx.clone();
        move |query| {
            tx.send(Event::SetCommandSearch(query.into())).unwrap();
        }
    });
    ui.global::<PaletteSearch>().on_reset_search({
        let tx = tx.clone();
        move || {
            tx.send(Event::SetCommandSearch("".into())).unwrap();
        }
    });

    ui.global::<PaletteSearch>().on_add_node({
        move |id| {
            tx.send(Event::AddNode(id.clone().into())).unwrap();
        }
    });
}

fn refresh(model: &Model, ui: &View) {
    let command_search = model.command_search();
    if command_search.is_empty() {
        ui.set_command_palette_results(VecModel::from_slice(&[]))
    } else {
        let project = model.tabs().selected_project();
        let res = project
            .search_available_nodes(command_search)
            .into_iter()
            .map(|(id, node)| SearchItem {
                id: id.0.clone().into(),
                category: node.category.as_str().into(),
                description: node.description.as_str().into(),
                name: node.name.as_str().into(),
            })
            .collect::<Vec<_>>();
        ui.set_command_palette_results(ModelRc::new(VecModel::from_slice(&res)))
    }
}

pub fn notify(ui: &View, model: &Model, evt: &Event) {
    use Event::*;
    match evt {
        AddNode(..) | SetNodePosition(..) | RemoveLink(..) | AddLink(..) => {}
        SelectTab(..) | NewTab | SetCommandSearch(..) => {
            refresh(model, ui);
        }
    }
}
