mod cli;
mod project;

use anyhow::{Ok, Result};
use clap::Parser;
use cli::Cli;
use std::process::exit;

fn main() {
    if let Err(err) = inner_main() {
        eprintln!("Error: {err}");

        exit(1);
    }
}

fn inner_main() -> Result<()> {
    let args = Cli::parse();

    let dir = &args.dir;

    let project_filter = args.project_filter();

    println!("Cleaning directory: {}", dir.display());
    println!("Project filter: {project_filter:?}");

    Ok(())
}
