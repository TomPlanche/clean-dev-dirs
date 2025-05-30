//! This build script is responsible for initializing the hooksmith configuration.
use std::path::Path;

fn main() {
    //  Rerun if the hooksmith configuration file has changed.
    //  @see https://doc.rust-lang.org/cargo/reference/build-scripts.html
    println!("cargo:rerun-if-changed=hooksmith.yaml");

    if let Err(err) = inner_main() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let hooksmith_config_path = Path::new("hooksmith.yaml");

    hooksmith::init(hooksmith_config_path)?;

    Ok(())
}
