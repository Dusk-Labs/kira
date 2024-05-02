mod model;
mod pres;
mod ui {
    slint::include_modules!();
}

use model::Model;
use slint::ComponentHandle;
use std::rc::Rc;
use ui::View;

fn main() -> Result<(), slint::PlatformError> {
    let mut model = Model::new();
    let ui = Rc::new(View::new()?);
    pres::setup(ui.clone(), &mut model);
    ui.run()
}
