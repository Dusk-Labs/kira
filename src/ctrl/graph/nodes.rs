use super::Aro;
use super::Controller;
use crate::ctrl::Event;
use crate::model::Model;
use crate::ui::Field;
use crate::ui::NodeData;
use crate::ui::Slot;
use crate::ui::View;
use crate::ui::{self};
use slint::ComponentHandle;
use slint::SharedString;
use slint::VecModel;
use std::sync::mpsc::Sender;

pub struct Nodes;

impl Controller for Nodes {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        // TODO: Make me a macro!
        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>()
            .on_move_node(move |node_idx, x, y| {
                tx_clone
                    .send(Event::SetNodePosition(node_idx as usize, x, y))
                    .unwrap();
            });

        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>()
            .on_set_field(move |node_idx, input, ty, value| {
                tx_clone
                    .send(Event::SetField {
                        node_idx: node_idx as _,
                        input: input.into(),
                        ty: ty.into(),
                        value: value.into(),
                    })
                    .unwrap();
            });

        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>()
            .on_set_field_random(move |node_idx, input| {
                tx_clone
                    .send(Event::SetFieldRandom {
                        node_idx: node_idx as _,
                        input: input.into(),
                    })
                    .unwrap();
            });

        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>()
            .on_set_field_inc(move |node_idx, input| {
                tx_clone
                    .send(Event::SetFieldInc {
                        node_idx: node_idx as _,
                        input: input.into(),
                    })
                    .unwrap();
            });

        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>()
            .on_set_field_dec(move |node_idx, input| {
                tx_clone
                    .send(Event::SetFieldDec {
                        node_idx: node_idx as _,
                        input: input.into(),
                    })
                    .unwrap();
            });

        let tx_clone = tx.clone();
        ui.global::<ui::NodeLogic>().on_focus_callback(move || {
            tx_clone.send(Event::FocusPalette).unwrap();
        });

        let model = model.read();
        refresh(ui, &model);
    }

    fn notify(ui: &View, model: &Model, _evt: &Event) {
        refresh(ui, model);
    }
}

fn refresh(ui: &View, model: &Model) {
    let Some(project) = model.tabs().selected_project() else {
        return;
    };

    let nodes = project
        .graph()
        .get_nodes()
        .iter()
        .enumerate()
        .map(|(idx, ni)| {
            let n = project.get_available_node(&ni.ty).unwrap();

            let inputs = n
                .inputs
                .iter()
                .map(|(name, ty)| Slot {
                    name: name.clone().into(),
                    ty: ty.clone().into(),
                })
                .collect::<Vec<_>>();

            let outputs = n
                .outputs
                .iter()
                .map(|(name, ty)| Slot {
                    name: name.clone().into(),
                    ty: ty.clone().into(),
                })
                .collect::<Vec<_>>();

            let input_fields = project.graph().get_state(idx).unwrap();
            let mut input_fields = input_fields
                .iter()
                .map(|(name, ty)| {
                    let options = ty
                        .options()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|text| SharedString::from(text.as_str()))
                        .collect::<Vec<_>>();

                    let text = ty.text().unwrap_or_default();

                    Field {
                        name: name.clone().into(),
                        ty: ty.ty().into(),
                        options: VecModel::from_slice(&options),
                        default_text: SharedString::from(text.as_str()),
                        default_value: ty.float().unwrap_or(1.0),
                        default_option: ty.option().map(|x| x as i32).unwrap_or(-1),
                        image: Default::default(),
                    }
                })
                .collect::<Vec<_>>();

            if let Some(ref img) = ni.image {
                let pix_buf = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                    img.as_raw(),
                    img.width(),
                    img.height(),
                );

                let img = slint::Image::from_rgb8(pix_buf);
                input_fields.push(Field {
                    name: "Output".into(),
                    ty: "Kira__Reserved_Image".into(),
                    options: VecModel::from_slice(&[]),
                    default_text: "".into(),
                    default_value: 1.0,
                    default_option: -1,
                    image: img,
                });
            }

            NodeData {
                inputs: VecModel::from_slice(&inputs),
                outputs: VecModel::from_slice(&outputs),
                input_fields: VecModel::from_slice(&input_fields),
                text: n.name.clone().into(),
                width: 280.,
                x: ni.pos.0,
                y: ni.pos.1,
            }
        })
        .collect::<Vec<_>>();

    ui.set_nodes(VecModel::from_slice(&nodes))
}
