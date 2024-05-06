use super::{Aro, Controller};
use crate::{
    ctrl::Event,
    model::{self, Model},
    ui::{self, LinkData, View},
};
use slint::{ComponentHandle, VecModel};
use std::sync::mpsc::Sender;

pub struct Links;

impl Controller for Links {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        ui.set_floating(ui::FloatingLinkData {
            floating_state: ui::FloatingState::None,
            ..Default::default()
        });
        setup_link_logic(ui, model.clone(), tx);

        let model = model.read();
        refresh(ui, &model);
    }

    fn notify(ui: &View, model: &Model, evt: &Event) {
        use Event::*;
        match evt {
            SetNodePosition(..) | CloseTab(..) | SelectTab(..) | NewTab | RemoveLink(..)
            | AddLink(..) => {
                refresh(ui, model);
            }
            AddNode(..) | SetCommandSearch(..) => {}
        }
    }
}

fn refresh(ui: &View, model: &Model) {
    if let Some(project) = model.tabs().selected_project() {
        ui.set_links(VecModel::from_slice(
            &project
                .get_links()
                .iter()
                .map(|l| LinkData {
                    dst: l.dst_node as i32,
                    dst_slot: l.dst_slot as i32,
                    src: l.src_node as i32,
                    src_slot: l.src_slot as i32,
                    ty: l.ty.0.clone().into(),
                })
                .collect::<Vec<_>>(),
        ))
    }
}

fn setup_link_logic(ui: &View, model: Aro<Model>, tx: Sender<Event>) {
    ui.global::<ui::LinkLogic>().on_new_link_from_output({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            if let Some(project) = model.tabs().selected_project() {
                if let Some(slot_ty) = project
                    .get_node(node_idx as usize)
                    .and_then(|ni| project.get_available_node(&ni.ty))
                    .map(|n| n.outputs[slot_idx as usize].1.clone())
                {
                    for (i, link) in project.get_links().iter().enumerate() {
                        if link.src_node == node_idx as usize && link.src_slot == slot_idx as usize
                        {
                            ui.set_floating(ui::FloatingLinkData {
                                floating_state: ui::FloatingState::DstAttached,
                                node: link.dst_node as i32,
                                node_slot: link.dst_slot as i32,
                                ty: slot_ty.into(),
                                x: 0.,
                                y: 0.,
                            });
                            tx.send(Event::RemoveLink(i)).unwrap();
                            return;
                        }
                    }

                    ui.set_floating(ui::FloatingLinkData {
                        floating_state: ui::FloatingState::SrcAttached,
                        node: node_idx,
                        node_slot: slot_idx,
                        ty: slot_ty.into(),
                        x: 0.,
                        y: 0.,
                    });
                }
            }
        }
    });
    ui.global::<ui::LinkLogic>().on_new_link_from_input({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            if let Some(project) = model.tabs().selected_project() {
                if let Some(slot_ty) = project
                    .get_node(node_idx as usize)
                    .and_then(|ni| project.get_available_node(&ni.ty))
                    .map(|n| n.inputs[slot_idx as usize].1.clone())
                {
                    for (i, link) in project.get_links().iter().enumerate() {
                        if link.dst_node == node_idx as usize && link.dst_slot == slot_idx as usize
                        {
                            // let link = links.remove(i);
                            ui.set_floating(ui::FloatingLinkData {
                                floating_state: ui::FloatingState::SrcAttached,
                                node: link.src_node as i32,
                                node_slot: link.src_slot as i32,
                                ty: slot_ty.into(),
                                x: 0.,
                                y: 0.,
                            });
                            tx.send(Event::RemoveLink(i)).unwrap();
                            return;
                        }
                    }

                    ui.set_floating(ui::FloatingLinkData {
                        floating_state: ui::FloatingState::DstAttached,
                        node: node_idx,
                        node_slot: slot_idx,
                        ty: slot_ty.into(),
                        x: 0.,
                        y: 0.,
                    });
                }
            }
        }
    });
    ui.global::<ui::LinkLogic>().on_attach_link_to_input({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            if let Some(project) = model.tabs().selected_project() {
                if let Some(slot_ty) = project
                    .get_node(node_idx as usize)
                    .and_then(|ni| project.get_available_node(&ni.ty))
                    .map(|n| n.inputs[slot_idx as usize].1.clone())
                {
                    let floating = ui.get_floating();
                    if floating.ty.as_str() == slot_ty.0
                        && floating.floating_state == ui::FloatingState::SrcAttached
                    {
                        tx.send(Event::AddLink(model::project::Link {
                            dst_node: node_idx as usize,
                            dst_slot: slot_idx as usize,
                            src_node: floating.node as usize,
                            src_slot: floating.node_slot as usize,
                            ty: slot_ty.clone(),
                        }))
                        .unwrap();
                    }
                }
                ui.set_floating(ui::FloatingLinkData {
                    floating_state: ui::FloatingState::None,
                    ..Default::default()
                });
            }
        }
    });
    ui.global::<ui::LinkLogic>().on_attach_link_to_output({
        let ui = ui.as_weak();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            if let Some(project) = model.tabs().selected_project() {
                if let Some(slot_ty) = project
                    .get_node(node_idx as usize)
                    .and_then(|ni| project.get_available_node(&ni.ty))
                    .map(|n| n.outputs[slot_idx as usize].1.clone())
                {
                    let floating = ui.get_floating();
                    if floating.ty.as_str() == slot_ty.0
                        && floating.floating_state == ui::FloatingState::DstAttached
                    {
                        tx.send(Event::AddLink(model::project::Link {
                            src_node: node_idx as usize,
                            src_slot: slot_idx as usize,
                            dst_node: floating.node as usize,
                            dst_slot: floating.node_slot as usize,
                            ty: slot_ty.clone(),
                        }))
                        .unwrap();
                    }
                }
                ui.set_floating(ui::FloatingLinkData {
                    floating_state: ui::FloatingState::None,
                    ..Default::default()
                });
            }
        }
    });
}
