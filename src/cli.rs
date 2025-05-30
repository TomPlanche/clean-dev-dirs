use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct FilteringArgs {
    /// Ignore projects with a build dir size smaller than the specified value
    #[arg(short = 's', long, default_value = "0")]
    keep_size: String,

    /// Ignore projects that have been compiled in the last [DAYS] days
    #[arg(short = 'd', long, default_value = "0")]
    keep_days: u32,
}

#[derive(Parser)]
struct ExecutionArgs {
    /// Don't ask for confirmation; Just clean all detected projects
    #[arg(short = 'y', long)]
    yes: bool,

    /// Just collect the cleanable projects and list the reclaimable space
    #[arg(long)]
    dry_run: bool,

    /// Use interactive project selection
    #[arg(short = 'i', long)]
    interactive: bool,
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
struct ScanningArgs {
    /// The number of threads to use for directory scanning
    #[arg(short = 't', long, default_value = "0")]
    threads: usize,

    /// Show access errors that occur while scanning
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Directories to ignore by default
    #[arg(long, action = clap::ArgAction::Append)]
    ignore: Vec<PathBuf>,

    /// Directories to skip during scanning
    #[arg(long, action = clap::ArgAction::Append)]
    skip: Vec<PathBuf>,
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

    /// Execution options
    #[command(flatten)]
    execution: ExecutionArgs,

    /// Filtering options
    #[command(flatten)]
    filtering: FilteringArgs,

    /// Scanning options
    #[command(flatten)]
    scanning: ScanningArgs,
}

#[derive(Clone, Debug)] // TODO: Remove Debug
pub(crate) struct ExecutionOptions {
    pub(crate) dry_run: bool,
    pub(crate) interactive: bool,
    pub(crate) yes: bool,
}

#[derive(Clone, Debug)]
pub struct FilterOptions {
    pub keep_size: String,
    pub keep_days: u32,
    pub project_filter: ProjectFilter,
}

#[derive(Clone, Debug, Copy, PartialEq)] // TODO: Remove Debug
pub(crate) enum ProjectFilter {
    All,
    RustOnly,
    NodeOnly,
}

#[derive(Clone, Debug)] // TODO: Remove Debug
pub struct ScanOptions {
    pub verbose: bool,
    pub threads: usize,
    pub ignore: Vec<PathBuf>,
    pub skip: Vec<PathBuf>,
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

    pub(crate) fn execution_options(&self) -> ExecutionOptions {
        ExecutionOptions {
            dry_run: self.execution.dry_run,
            interactive: self.execution.interactive,
            yes: self.execution.yes,
        }
    }

    pub(crate) fn filter_options(&self) -> FilterOptions {
        FilterOptions {
            keep_size: self.filtering.keep_size.clone(),
            keep_days: self.filtering.keep_days,
            project_filter: self.project_filter(),
        }
    }

    pub(crate) fn scan_options(&self) -> ScanOptions {
        ScanOptions {
            verbose: self.scanning.verbose,
            threads: self.scanning.threads,
            ignore: self.scanning.ignore.clone(),
            skip: self.scanning.skip.clone(),
        }
    }
}
