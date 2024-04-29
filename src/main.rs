mod backend;
mod command_palette;
mod node_view;
mod ui {
    slint::include_modules!();
}

use crate::ui::AppWindow;
use command_palette::CommandPalette;
use node_view::NodeView;
use slint::ComponentHandle;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let node_view = Rc::new(NodeView::new(&ui));
    let command_palette = Rc::new(CommandPalette::new(&ui));

    node_view.setup(&ui);
    command_palette.setup(&ui, node_view.clone());

    ui.run()
}
