use crate::{
    model::Project,
    ui::{Item, PaletteSearch, View},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::sync::{mpsc::Sender, Arc, RwLock};

use super::Event;

pub fn setup(project: Arc<RwLock<Project>>, ui: &View, tx: Sender<Event>) {
    ui.global::<PaletteSearch>().on_search({
        let project = project.clone();
        move |query| {
            let results = project
                .read()
                .unwrap()
                .search_available_nodes(&query)
                .into_iter()
                .map(|(id, node)| Item {
                    id: id.0.clone().into(),
                    category: node.category.as_str().into(),
                    description: node.description.as_str().into(),
                    name: node.name.as_str().into(),
                })
                .collect::<Vec<_>>();

            ModelRc::new(VecModel::from(results))
        }
    });

    ui.global::<PaletteSearch>().on_add_node({
        let project = project.clone();
        move |id| {
            if project
                .read()
                .unwrap()
                .get_available_node(&id.clone().into())
                .is_some()
            {
                tx.send(Event::AddNode(id.clone().into())).unwrap();
            }
        }
    });
}
