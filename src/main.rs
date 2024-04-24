use slint::Model;
use slint::VecModel;
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let nodes = Rc::new(VecModel::from(vec![
        Node {
            text: "Node1 Lorem ipsum dolor sit amet".into(),
            x: 120.,
            y: 235.,
            width: 90.,
            inputs: [LinkType::A].into(),
            outputs: [LinkType::B, LinkType::B].into(),
        },
        Node {
            text: "Node2".into(),
            x: 350.,
            y: 150.,
            width: 90.,
            inputs: [].into(),
            outputs: [LinkType::A].into(),
        },
        Node {
            text: "Node3".into(),
            x: 300.,
            y: 350.,
            width: 90.,
            inputs: [LinkType::A, LinkType::B].into(),
            outputs: [].into(),
        },
        Node {
            text: "Node4".into(),
            x: 400.,
            y: 450.,
            width: 90.,
            inputs: [LinkType::A, LinkType::B].into(),
            outputs: [LinkType::A, LinkType::B].into(),
        },
    ]));
    let links = Rc::from(VecModel::from(vec![
        LinkData {
            src: 1,
            src_slot: 0,
            dst: 2,
            dst_slot: 0,
            ty: LinkType::A,
            detached_src: false,
            detached_dst: false,
            detached_x: 0.,
            detached_y: 0.,
        },
        LinkData {
            src: 0,
            src_slot: 0,
            dst: 2,
            dst_slot: 1,
            ty: LinkType::B,
            detached_src: false,
            detached_dst: false,
            detached_x: 0.,
            detached_y: 0.,
        },
        LinkData {
            src: 0,
            src_slot: 1,
            dst: 2,
            dst_slot: 1,
            ty: LinkType::B,
            detached_src: false,
            detached_dst: false,
            detached_x: 0.,
            detached_y: 0.,
        },
    ]));

    let ui = AppWindow::new()?;
    ui.set_nodes(nodes.clone().into());
    ui.set_links(links.clone().into());

    ui.global::<LinkLogic>().on_try_attach_src({
        let nodes = nodes.clone();
        move |mut link| {
            for (node_idx, node) in nodes.iter().enumerate() {
                for (slot_idx, slot) in node.outputs.iter().enumerate() {
                    let has_collision = {
                        let slot_idx = f32::from(u8::try_from(slot_idx).unwrap());
                        let detach_top = link.detached_y;
                        let detach_bottom = link.detached_y + 10.;
                        let detach_left = link.detached_x;
                        let detach_right = link.detached_x + 10.;
                        let output_top = node.y + 15. * slot_idx;
                        let output_bottom = node.y + 15. * slot_idx + 10.;
                        let output_left = node.x + node.width;
                        let output_right = node.x + node.width + 10.;

                        detach_left < output_right
                            && detach_right > output_left
                            && detach_top < output_bottom
                            && detach_bottom > output_top
                    };

                    if has_collision && slot == link.ty {
                        link.detached_src = false;
                        link.src = node_idx.try_into().unwrap();
                        link.src_slot = slot_idx.try_into().unwrap();
                        return link;
                    }
                }
            }
            link.detached_src = false;
            link
        }
    });
    ui.global::<LinkLogic>().on_try_attach_dst({
        let nodes = nodes.clone();
        move |mut link| {
            for (node_idx, node) in nodes.iter().enumerate() {
                for (slot_idx, slot) in node.inputs.iter().enumerate() {
                    let has_collision = {
                        let slot_idx = f32::from(u8::try_from(slot_idx).unwrap());
                        let detach_top = link.detached_y;
                        let detach_bottom = link.detached_y + 10.;
                        let detach_left = link.detached_x;
                        let detach_right = link.detached_x + 10.;
                        let input_top = node.y + 15. * slot_idx;
                        let input_bottom = node.y + 15. * slot_idx + 10.;
                        let input_left = node.x - 10.;
                        let input_right = node.x;

                        detach_left < input_right
                            && detach_right > input_left
                            && detach_top < input_bottom
                            && detach_bottom > input_top
                    };

                    if has_collision && slot == link.ty {
                        link.detached_dst = false;
                        link.dst = node_idx.try_into().unwrap();
                        link.dst_slot = slot_idx.try_into().unwrap();
                        return link;
                    }
                }
            }
            link.detached_dst = false;
            link
        }
    });

    ui.run()
}
