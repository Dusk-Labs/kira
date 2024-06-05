use std::collections::HashMap;
use std::process::Command;

fn main() {
    println!("cargo:rustc-env=SLINT_ENABLE_EXPERIMENTAL_FEATURES=1");

    let libs = HashMap::from([("ui".into(), "ui/".into())]);
    let conf = slint_build::CompilerConfiguration::new().with_library_paths(libs);
    slint_build::compile_with_config("ui/app.slint", conf).unwrap();

    let _ = Command::new("git")
        .args(&["rev-parse", "--short=8", "HEAD"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(output.stdout.as_ref())
                .trim()
                .to_string()
        })
        .map(|hash| println!("cargo:rustc-env=GIT_COMMIT_HASH={}", hash));
}
