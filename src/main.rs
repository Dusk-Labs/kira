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
    let ui = View::new()?;
    let ctrl = Mediator::new(&ui, model);

    std::thread::spawn({
        let ui = ui.as_weak();
        move || {
            ctrl.run(ui);
        }
    });
    ui.run()
}
