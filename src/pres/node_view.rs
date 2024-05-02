use crate::{
    model,
    model::Project,
    ui,
    ui::{LinkData, Node, View},
};
use slint::{ComponentHandle, LogicalPosition, VecModel};
use std::{cell::RefCell, rc::Rc};

pub fn setup(ui: Rc<View>, project: Rc<RefCell<Project>>) {
    {
        let mut project = (*project).borrow_mut();

        refresh(ui.clone(), &project);

        project.subscribe(Box::new({
            let ui = ui.clone();
            move |prj: &Project| refresh(ui.clone(), prj)
        }));
    }

    ui.set_floating(ui::FloatingLinkData {
        floating_state: ui::FloatingState::None,
        ..Default::default()
    });

    setup_node_logic(ui.clone(), project.clone());
    setup_link_logic(ui.clone(), project);
    setup_move_area_logic(ui);
}

fn refresh(ui: Rc<View>, project: &Project) {
    refresh_ui_links(ui.clone(), project);
    refresh_ui_nodes(ui, project);
}

fn refresh_ui_links(ui: Rc<View>, project: &Project) {
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
    ));
}

fn refresh_ui_nodes(ui: Rc<View>, project: &Project) {
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
                            .map(|(_, lt)| lt.clone().into())
                            .collect::<Vec<_>>(),
                    ),
                    outputs: VecModel::from_slice(
                        &n.outputs
                            .iter()
                            .map(|(_, lt)| lt.clone().into())
                            .collect::<Vec<_>>(),
                    ),
                    text: n.name.clone().into(),
                    width: 90.,
                    x: ni.pos.0,
                    y: ni.pos.1,
                }
            })
            .collect::<Vec<_>>(),
    ));
}

fn setup_link_logic(ui: Rc<View>, project: Rc<RefCell<Project>>) {
    ui.global::<ui::LinkLogic>().on_new_link_from_output({
        let ui = ui.clone();
        let project = project.clone();
        move |node_idx, slot_idx| {
            let mut project = (*project).borrow_mut();
            if let Some(slot_ty) = project
                .get_node(node_idx as usize)
                .and_then(|ni| project.get_available_node(&ni.ty))
                .map(|n| n.outputs[slot_idx as usize].1.clone())
            {
                for (i, link) in project.get_links().iter().enumerate() {
                    if link.src_node == node_idx as usize && link.src_slot == slot_idx as usize {
                        // let link = links.remove(i);
                        ui.set_floating(ui::FloatingLinkData {
                            floating_state: ui::FloatingState::DstAttached,
                            node: link.dst_node as i32,
                            node_slot: link.dst_slot as i32,
                            ty: slot_ty.into(),
                            x: 0.,
                            y: 0.,
                        });
                        project.remove_link(i);
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
        let ui = ui.clone();
        let project = project.clone();
        move |node_idx, slot_idx| {
            let mut project = (*project).borrow_mut();
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
                        project.remove_link(i);
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
        let ui = ui.clone();
        let project = project.clone();
        move |node_idx, slot_idx| {
            let mut project = (*project).borrow_mut();
            if let Some(slot_ty) = project
                .get_node(node_idx as usize)
                .and_then(|ni| project.get_available_node(&ni.ty))
                .map(|n| n.inputs[slot_idx as usize].1.clone())
            {
                let floating = ui.get_floating();
                if floating.ty.as_str() == slot_ty.0
                    && floating.floating_state == ui::FloatingState::SrcAttached
                {
                    project.add_link(model::project::Link {
                        dst_node: node_idx as usize,
                        dst_slot: slot_idx as usize,
                        src_node: floating.node as usize,
                        src_slot: floating.node_slot as usize,
                        ty: slot_ty.clone(),
                    });
                }
            }
            ui.set_floating(ui::FloatingLinkData {
                floating_state: ui::FloatingState::None,
                ..Default::default()
            });
        }
    });
    ui.global::<ui::LinkLogic>().on_attach_link_to_output({
        let ui = ui.clone();
        let project = project.clone();
        move |node_idx, slot_idx| {
            let mut project = (*project).borrow_mut();
            if let Some(slot_ty) = project
                .get_node(node_idx as usize)
                .and_then(|ni| project.get_available_node(&ni.ty))
                .map(|n| n.outputs[slot_idx as usize].1.clone())
            {
                let floating = ui.get_floating();
                if floating.ty.as_str() == slot_ty.0
                    && floating.floating_state == ui::FloatingState::DstAttached
                {
                    project.add_link(model::project::Link {
                        src_node: node_idx as usize,
                        src_slot: slot_idx as usize,
                        dst_node: floating.node as usize,
                        dst_slot: floating.node_slot as usize,
                        ty: slot_ty.clone(),
                    });
                }
            }
            ui.set_floating(ui::FloatingLinkData {
                floating_state: ui::FloatingState::None,
                ..Default::default()
            });
        }
    });
}

fn setup_move_area_logic(ui: Rc<View>) {
    ui.global::<ui::MoveAreaLogic>().on_click_event_hack({
        let ui = ui.as_weak();
        move |x, y, evt| {
            let ui = ui.upgrade().unwrap();
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::PointerPressed {
                    position: LogicalPosition { x, y },
                    button: evt.button,
                });
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::PointerReleased {
                    position: LogicalPosition { x, y },
                    button: evt.button,
                });
        }
    });
    ui.global::<ui::MoveAreaLogic>()
        .on_update_floating_position({
            let ui = ui.as_weak();
            move |x, y| {
                let ui = ui.upgrade().unwrap();
                ui.set_floating(ui::FloatingLinkData {
                    x,
                    y,
                    ..ui.get_floating()
                });
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

fn setup_node_logic(ui: Rc<View>, project: Rc<RefCell<Project>>) {
    ui.global::<ui::NodeLogic>().on_move_node({
        let project = project.clone();
        move |node_idx, x, y| {
            let mut project = (*project).borrow_mut();
            project.set_node_position(node_idx as usize, x, y);
        }
    });
}
