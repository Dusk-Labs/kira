use super::{Aro, Controller};
use crate::{
    ctrl::Event,
    model::{self, Model, Project},
    ui::{self, LinkData, Node, Slot, View},
};
use slint::{ComponentHandle, LogicalPosition, VecModel};
use std::sync::mpsc::Sender;

pub struct NodeView;

impl Controller for NodeView {
    fn setup(model: Aro<Model>, ui: &View, tx: Sender<Event>) {
        ui.set_floating(ui::FloatingLinkData {
            floating_state: ui::FloatingState::None,
            ..Default::default()
        });

        setup_node_logic(ui, tx.clone());
        setup_link_logic(ui, model.clone(), tx);
        setup_move_area_logic(ui);

        let model = model.read();
        let project = model.tabs().selected_project();
        refresh_ui_links(ui, project);
        refresh_ui_nodes(ui, project);
    }

    fn notify(ui: &View, model: &Model, evt: &Event) {
        use Event::*;
        let project = model.tabs().selected_project();
        match evt {
            SetNodePosition(..) | SelectTab(..) | NewTab => {
                refresh_ui_nodes(ui, project);
                refresh_ui_links(ui, project);
            }
            RemoveLink(..) | AddLink(..) => {
                refresh_ui_links(ui, project);
            }
            AddNode(..) => {
                refresh_ui_nodes(ui, project);
            }
            SetCommandSearch(..) => {}
        }
    }
}

fn refresh_ui_links(ui: &View, project: &Project) {
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

fn refresh_ui_nodes(ui: &View, project: &Project) {
    ui.set_nodes(VecModel::from_slice(
        &project
            .get_nodes()
            .iter()
            .map(|ni| {
                let n = project.get_available_node(&ni.ty).unwrap();
                Node {
                    inputs: VecModel::from_slice(
                        &n.inputs
                            .iter()
                            .map(|(name, ty)| Slot {
                                name: name.clone().into(),
                                ty: ty.clone().into(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    outputs: VecModel::from_slice(
                        &n.outputs
                            .iter()
                            .map(|(name, ty)| Slot {
                                name: name.clone().into(),
                                ty: ty.clone().into(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    text: n.name.clone().into(),
                    width: 100.,
                    x: ni.pos.0,
                    y: ni.pos.1,
                }
            })
            .collect::<Vec<_>>(),
    ))
}
fn setup_link_logic(ui: &View, model: Aro<Model>, tx: Sender<Event>) {
    ui.global::<ui::LinkLogic>().on_new_link_from_output({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            let project = model.tabs().selected_project();
            if let Some(slot_ty) = project
                .get_node(node_idx as usize)
                .and_then(|ni| project.get_available_node(&ni.ty))
                .map(|n| n.outputs[slot_idx as usize].1.clone())
            {
                for (i, link) in project.get_links().iter().enumerate() {
                    if link.src_node == node_idx as usize && link.src_slot == slot_idx as usize {
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
    });
    ui.global::<ui::LinkLogic>().on_new_link_from_input({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            let project = model.tabs().selected_project();
            if let Some(slot_ty) = project
                .get_node(node_idx as usize)
                .and_then(|ni| project.get_available_node(&ni.ty))
                .map(|n| n.inputs[slot_idx as usize].1.clone())
            {
                for (i, link) in project.get_links().iter().enumerate() {
                    if link.dst_node == node_idx as usize && link.dst_slot == slot_idx as usize {
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
    });
    ui.global::<ui::LinkLogic>().on_attach_link_to_input({
        let ui = ui.as_weak();
        let tx = tx.clone();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            let project = model.tabs().selected_project();
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
    });
    ui.global::<ui::LinkLogic>().on_attach_link_to_output({
        let ui = ui.as_weak();
        let model = model.clone();
        move |node_idx, slot_idx| {
            let ui = ui.upgrade().unwrap();
            let model = model.read();
            let project = model.tabs().selected_project();
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
    });
}

fn setup_move_area_logic(ui: &View) {
    ui.global::<ui::MoveAreaLogic>().on_mouse_event_tap_hack({
        let ui = ui.as_weak();
        move |abs_x, abs_y, rel_x, rel_y, evt| {
            let ui = ui.upgrade().unwrap();

            let position = LogicalPosition { x: abs_x, y: abs_y };
            let button = evt.button;

            use slint::private_unstable_api::re_exports::PointerEventKind::*;
            match evt.kind {
                Up => {
                    ui.window()
                        .dispatch_event(slint::platform::WindowEvent::PointerReleased {
                            position,
                            button,
                        });
                }
                Down => ui
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::PointerPressed {
                        position,
                        button,
                    }),
                Move => {
                    ui.set_floating(ui::FloatingLinkData {
                        x: rel_x,
                        y: rel_y,
                        ..ui.get_floating()
                    });

                    // FIXME
                    // ui.window()
                    //     .dispatch_event(slint::platform::WindowEvent::PointerMoved { position })
                }
                Cancel => ui
                    .window()
                    .dispatch_event(slint::platform::WindowEvent::PointerExited {}),
            }
        }
    });
    ui.global::<ui::MoveAreaLogic>().on_reset_floating_state({
        let ui = ui.as_weak();
        move || {
            let ui = ui.upgrade().unwrap();
            ui.set_floating(ui::FloatingLinkData {
                floating_state: ui::FloatingState::None,
                ..Default::default()
            });
        }
    });
}

fn setup_node_logic(ui: &View, tx: Sender<Event>) {
    ui.global::<ui::NodeLogic>().on_move_node({
        move |node_idx, x, y| {
            tx.send(Event::SetNodePosition(node_idx as usize, x, y))
                .unwrap();
        }
    });
}
