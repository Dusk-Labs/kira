use crate::{
    model::Project,
    ui::{Item, PaletteSearch, View},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

pub fn setup(ui: Rc<View>, project: Rc<RefCell<Project>>) {
    ui.global::<PaletteSearch>().on_search({
        let project = project.clone();
        move |query| {
            let project = (*project).borrow();
            let results = project
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
            let mut project = (*project).borrow_mut();
            if project.get_available_node(&id.clone().into()).is_some() {
                project.add_node(id.clone().into());
            }
        }
    });
}
