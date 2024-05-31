use std::collections::HashMap;

fn main() {
    let libs = HashMap::from([("ui".into(), "ui/".into())]);

    println!("cargo:rustc-env=SLINT_ENABLE_EXPERIMENTAL_FEATURES=1");

    let conf = slint_build::CompilerConfiguration::new().with_library_paths(libs);

    slint_build::compile_with_config("ui/app.slint", conf).unwrap();
}
