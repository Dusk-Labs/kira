mod ctrl;
mod model;
mod utils;
mod ui {
    slint::include_modules!();
}

use slint::ComponentHandle;

use ctrl::Mediator;
use model::Model;
use ui::View;

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();

    let model = Model::new();

    let ui = View::new()?;
    let ctrl = Mediator::new(&ui, model);

    std::thread::spawn(move || {
        ctrl.run();
    });

    ui.run()
}
