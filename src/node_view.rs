use crate::ui::*;
use slint::LogicalPosition;
use slint::Model;
use slint::VecModel;
use std::rc::Rc;

pub struct NodeView {
    nodes: Rc<VecModel<Node>>,
    links: Rc<VecModel<LinkData>>,
}

impl NodeView {
    pub fn add_node(&self, node: Node) {
        self.nodes.push(node);
    }
    pub fn new(ui: &AppWindow) -> Self {
        let node_view = Self {
            nodes: Rc::new(VecModel::from(vec![])),
            links: Rc::new(VecModel::from(vec![])),
        };

        ui.set_nodes(node_view.nodes.clone().into());
        ui.set_links(node_view.links.clone().into());
        ui.set_floating(FloatingLinkData {
            floating_state: FloatingState::None,
            ..Default::default()
        });

        node_view
    }

    pub fn setup(&self, ui: &AppWindow) {
        self.setup_link_logic(ui);
        self.setup_move_area_logic(ui);
    }

    fn setup_link_logic(&self, ui: &AppWindow) {
        ui.global::<LinkLogic>().on_new_link_from_output({
            let nodes = self.nodes.clone();
            let links = self.links.clone();
            let ui = ui.as_weak();
            move |node_idx, slot_idx| {
                if let Some(slot_ty) = nodes
                    .row_data(node_idx as usize)
                    .and_then(|n| n.outputs.row_data(slot_idx as usize))
                {
                    for i in 0..links.row_count() {
                        if let Some(link) = links.row_data(i) {
                            if link.src == node_idx && link.src_slot == slot_idx {
                                let link = links.remove(i);
                                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                                    floating_state: FloatingState::DstAttached,
                                    node: link.dst,
                                    node_slot: link.dst_slot,
                                    ty: slot_ty,
                                    x: 0.,
                                    y: 0.,
                                });
                                return;
                            }
                        }
                    }

                    ui.upgrade().unwrap().set_floating(FloatingLinkData {
                        floating_state: FloatingState::SrcAttached,
                        node: node_idx,
                        node_slot: slot_idx,
                        ty: slot_ty,
                        x: 0.,
                        y: 0.,
                    });
                }
            }
        });
        ui.global::<LinkLogic>().on_new_link_from_input({
            let nodes = self.nodes.clone();
            let links = self.links.clone();
            let ui = ui.as_weak();
            move |node_idx, slot_idx| {
                if let Some(slot_ty) = nodes
                    .row_data(node_idx as usize)
                    .and_then(|n| n.inputs.row_data(slot_idx as usize))
                {
                    for i in 0..links.row_count() {
                        if let Some(link) = links.row_data(i) {
                            if link.dst == node_idx && link.dst_slot == slot_idx {
                                let link = links.remove(i);
                                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                                    floating_state: FloatingState::SrcAttached,
                                    node: link.src,
                                    node_slot: link.src_slot,
                                    ty: slot_ty,
                                    x: 0.,
                                    y: 0.,
                                });
                                return;
                            }
                        }
                    }
                    ui.upgrade().unwrap().set_floating(FloatingLinkData {
                        floating_state: FloatingState::DstAttached,
                        node: node_idx,
                        node_slot: slot_idx,
                        ty: slot_ty,
                        x: 0.,
                        y: 0.,
                    });
                }
            }
        });
        ui.global::<LinkLogic>().on_attach_link_to_input({
            let nodes = self.nodes.clone();
            let links = self.links.clone();
            let ui = ui.as_weak();
            move |node_idx, slot_idx| {
                if let Some(slot_ty) = nodes
                    .row_data(node_idx as usize)
                    .and_then(|n| n.inputs.row_data(slot_idx as usize))
                {
                    let floating = ui.upgrade().unwrap().get_floating();
                    if floating.ty == slot_ty
                        && floating.floating_state == FloatingState::SrcAttached
                    {
                        links.push(LinkData {
                            dst: node_idx,
                            dst_slot: slot_idx,
                            src: floating.node,
                            src_slot: floating.node_slot,
                            ty: floating.ty,
                        });
                    }
                }
                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                    floating_state: FloatingState::None,
                    ..Default::default()
                });
            }
        });
        ui.global::<LinkLogic>().on_attach_link_to_output({
            let nodes = self.nodes.clone();
            let links = self.links.clone();
            let ui = ui.as_weak();
            move |node_idx, slot_idx| {
                if let Some(slot_ty) = nodes
                    .row_data(node_idx as usize)
                    .and_then(|n| n.outputs.row_data(slot_idx as usize))
                {
                    let floating = ui.upgrade().unwrap().get_floating();
                    if floating.ty == slot_ty
                        && floating.floating_state == FloatingState::DstAttached
                    {
                        links.push(LinkData {
                            src: node_idx,
                            src_slot: slot_idx,
                            dst: floating.node,
                            dst_slot: floating.node_slot,
                            ty: floating.ty,
                        });
                    }
                }
                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                    floating_state: FloatingState::None,
                    ..Default::default()
                });
            }
        });
    }

    fn setup_move_area_logic(&self, ui: &AppWindow) {
        ui.global::<MoveAreaLogic>().on_click_event_hack({
            let ui = ui.as_weak();
            move |x, y, evt| {
                ui.upgrade().unwrap().window().dispatch_event(
                    slint::platform::WindowEvent::PointerPressed {
                        position: LogicalPosition { x, y },
                        button: evt.button,
                    },
                );
                ui.upgrade().unwrap().window().dispatch_event(
                    slint::platform::WindowEvent::PointerReleased {
                        position: LogicalPosition { x, y },
                        button: evt.button,
                    },
                );
            }
        });
        ui.global::<MoveAreaLogic>().on_update_floating_position({
            let ui = ui.as_weak();
            move |x, y| {
                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                    x,
                    y,
                    ..ui.upgrade().unwrap().get_floating()
                });
            }
        });
        ui.global::<MoveAreaLogic>().on_reset_floating_state({
            let ui = ui.as_weak();
            move || {
                ui.upgrade().unwrap().set_floating(FloatingLinkData {
                    floating_state: FloatingState::None,
                    ..Default::default()
                });
            }
        });
    }
}

// let nodes =
//     Node {
//         text: "Node1 Lorem ipsum dolor sit amet".into(),
//         x: 120.,
//         y: 235.,
//         width: 90.,
//         inputs: [LinkType::A].into(),
//         outputs: [LinkType::B, LinkType::B].into(),
//     },
//     Node {
//         text: "Node2".into(),
//         x: 350.,
//         y: 150.,
//         width: 90.,
//         inputs: [].into(),
//         outputs: [LinkType::A].into(),
//     },
//     Node {
//         text: "Node3".into(),
//         x: 300.,
//         y: 350.,
//         width: 90.,
//         inputs: [LinkType::A, LinkType::B].into(),
//         outputs: [].into(),
//     },
//     Node {
//         text: "Node4".into(),
//         x: 400.,
//         y: 450.,
//         width: 90.,
//         inputs: [LinkType::A, LinkType::B].into(),
//         outputs: [LinkType::A, LinkType::B].into(),
//     },
// ]));
// let links = Rc::from(VecModel::from(vec![
//     LinkData {
//         src: 1,
//         src_slot: 0,
//         dst: 2,
//         dst_slot: 0,
//         ty: LinkType::A,
//     },
//     LinkData {
//         src: 0,
//         src_slot: 0,
//         dst: 2,
//         dst_slot: 1,
//         ty: LinkType::B,
//     },
//     LinkData {
//         src: 0,
//         src_slot: 1,
//         dst: 2,
//         dst_slot: 1,
//         ty: LinkType::B,
//     },
// ]));
// let floating = FloatingLinkData {
//     floating_state: FloatingState::None,
//     ..Default::default()
// };
