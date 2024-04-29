use std::collections::HashMap;
use std::rc::Rc;

use crate::backend;
use crate::node_view::NodeView;
use crate::ui::*;
use simsearch::SimSearch;
use slint::ModelRc;
use slint::SharedString;
use slint::VecModel;

pub struct CommandPalette {
    index: Rc<SimSearch<String>>,
    nodes: Rc<HashMap<String, backend::Node>>,
}

impl CommandPalette {
    pub fn new(_ui: &AppWindow) -> Self {
        let nodes = backend::nodes();
        let cmd = Self {
            index: Rc::new(backend::build_index(&nodes)),
            nodes: Rc::new(nodes),
        };
        cmd
    }
    pub fn setup(&self, ui: &AppWindow, node_view: Rc<NodeView>) {
        ui.global::<PaletteSearch>().on_search({
            let index = self.index.clone();
            let nodes = self.nodes.clone();
            move |query| {
                let ids = index.search(&*query);
                let results = ids
                    .into_iter()
                    .take(6)
                    .filter_map(|id| nodes.get_key_value(id.as_str()))
                    .map(|(id, node)| Item {
                        id: id.into(),
                        category: node.category.as_str().into(),
                        description: node.description.as_str().into(),
                        name: node.display_name.as_str().into(),
                    })
                    .collect::<Vec<_>>();

                ModelRc::new(VecModel::from(results))
            }
        });

        ui.global::<PaletteSearch>().on_add_node({
            let nodes = self.nodes.clone();
            let node_view = node_view.clone();
            move |id| {
                if let Some(n) = nodes.get(id.as_str()) {
                    node_view.add_node(Node {
                        inputs: ModelRc::new(VecModel::from(
                            n.input
                                .values()
                                .cloned()
                                .map(SharedString::from)
                                .collect::<Vec<_>>(),
                        )),
                        outputs: ModelRc::new(VecModel::from(
                            n.output.iter().map(SharedString::from).collect::<Vec<_>>(),
                        )),
                        text: n.display_name.clone().into(),
                        width: 90.,
                        x: 50.,
                        y: 50.,
                    });
                }
            }
        });
    }
}
