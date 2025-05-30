use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq)] // TODO: Remove Debug
pub enum ProjectFilter {
    All,
    RustOnly,
    NodeOnly,
}

#[derive(Parser)]
struct ProjectTypeArgs {
    /// Clean only Rust projects
    #[arg(long, conflicts_with = "node_only")]
    rust_only: bool,

    /// Clean only Node.js projects
    #[arg(long, conflicts_with = "rust_only")]
    node_only: bool,
}

#[derive(Parser)]
#[command(name = "clean-dev-dirs")]
#[command(about = "Recursively clean all Rust target and Node.js node_modules directories")]
pub(crate) struct Cli {
    /// The directory to search for projects
    #[arg(default_value = ".")]
    pub(crate) dir: PathBuf,

    /// Project type to clean
    #[command(flatten)]
    project_type: ProjectTypeArgs,
}

impl Cli {
    pub(crate) fn project_filter(&self) -> ProjectFilter {
        if self.project_type.rust_only {
            ProjectFilter::RustOnly
        } else if self.project_type.node_only {
            ProjectFilter::NodeOnly
        } else {
            ProjectFilter::All
        }
    }
}
