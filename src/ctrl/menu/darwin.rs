use crate::ctrl::Controller;
use crate::ctrl::Event;
use crate::ui::View;
use crate::utils::Aro;
use crate::Model;

use muda::AboutMetadata;
use muda::AboutMetadataBuilder;
use muda::Menu;
use muda::MenuItemBuilder;
use muda::PredefinedMenuItem;
use muda::Submenu;
use muda::SubmenuBuilder;

use std::sync::mpsc::Sender;

pub struct DarwinMenu;

impl Controller for DarwinMenu {
    fn setup(_: Aro<Model>, _: &View, tx: Sender<Event>) {
        let app = SubmenuBuilder::new()
            .enabled(true)
            .text("&Kira")
            .items(&[
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::about(None, Some(about())),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .build()
            .expect("Failed to build App submenu");

        let file = SubmenuBuilder::new()
            .enabled(true)
            .text("&File")
            .items(&[
                &MenuItemBuilder::new()
                    .enabled(true)
                    .text("Open Project...")
                    .id("open-file".into())
                    .acccelerator(Some("CMD+O"))
                    .unwrap()
                    .build(),
                &MenuItemBuilder::new()
                    .enabled(true)
                    .text("Save")
                    .id("save".into())
                    .acccelerator(Some("CMD+S"))
                    .unwrap()
                    .build(),
                &MenuItemBuilder::new()
                    .enabled(true)
                    .text("Save as...")
                    .id("save-as".into())
                    .acccelerator(Some("CMD+SHIFT+S"))
                    .unwrap()
                    .build(),
                &PredefinedMenuItem::separator(),
                &MenuItemBuilder::new()
                    .enabled(true)
                    .text("Open Command Palette...")
                    .id("command-palette".into())
                    .acccelerator(Some("CMD+K"))
                    .unwrap()
                    .build(),
                &MenuItemBuilder::new()
                    .enabled(true)
                    .text("Render Current Segment")
                    .id("render".into())
                    .acccelerator(Some("CMD+R"))
                    .unwrap()
                    .build(),
            ])
            .build()
            .unwrap();

        let menu_bar = Menu::new();

        menu_bar.append_items(&[&app, &file]).unwrap();
        menu_bar.init_for_nsapp();

        // NOTE: Very important, otherwise the whole app will crash when anything in the menu bar
        // gets clicked.
        core::mem::forget(menu_bar);
        core::mem::forget(app);
        core::mem::forget(file);

        let handle_event = move |event: muda::MenuEvent| match event.id.0.as_str() {
            "render" => tx.send(Event::Render).unwrap(),
            "command-palette" => tx.send(Event::TogglePalette).unwrap(),
            "open-file" => tx.send(Event::OpenFile).unwrap(),
            "save" => tx.send(Event::Save).unwrap(),
            "save-as" => tx.send(Event::SaveAs).unwrap(),
            unk => println!("Unknown menu-bar event: {}", unk),
        };

        muda::MenuEvent::set_event_handler(Some(handle_event));
    }

    fn notify(_: &View, _: &Model, _: &Event) {}
}

fn about() -> AboutMetadata {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png");
    let icon = image::open(path)
        .into_iter()
        .filter_map(|icon| {
            let icon = icon.into_rgba8();
            let width = icon.width();
            let height = icon.height();

            muda::Icon::from_rgba(icon.into_raw(), width, height).ok()
        })
        .next();

    let version = format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_HASH")
    );

    AboutMetadataBuilder::new()
        .icon(icon)
        .name(Some("Kira"))
        .version(Some(version))
        .copyright(Some("Copyright © 2024 Dusk Labs. All rights reserved."))
        .build()
}
