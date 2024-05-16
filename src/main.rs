mod ctrl;
mod model;
mod utils;
mod ui {
    slint::include_modules!();
}

use ctrl::Mediator;
use model::Model;
use slint::ComponentHandle;
use ui::View;

fn main() -> Result<(), slint::PlatformError> {
    let model = Model::new();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn({
        move || {
            let ui = View::new()?;
            let ctrl = Mediator::new(&ui, model);
            tx.send(ctrl).unwrap();
            ui.run()
        }
    });
    let ctrl = rx.recv().unwrap();
    ctrl.run();
    Ok(())
}
