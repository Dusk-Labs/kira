use super::Aro;
use super::Controller;
use crate::ctrl::Event;
use crate::model::Model;
use crate::ui::View;
use crate::ui::{self};
use slint::ComponentHandle;
use slint::LogicalPosition;
use std::sync::mpsc::Sender;

pub struct Floating;

impl Controller for Floating {
    fn setup(_model: Aro<Model>, ui: &View, _tx: Sender<Event>) {
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
                    Down => {
                        ui.window()
                            .dispatch_event(slint::platform::WindowEvent::PointerPressed {
                                position,
                                button,
                            })
                    }
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

    fn notify(_ui: &View, _model: &Model, _evt: &Event) {}
}
