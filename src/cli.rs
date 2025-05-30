//! Command-line interface definition and argument parsing.
//!
//! This module defines all command-line arguments, options, and their validation
//! using the [clap](https://docs.rs/clap/) library. It provides structured access
//! to user input and handles argument conflicts and defaults.

use std::path::PathBuf;

use clap::Parser;

/// Command-line arguments for filtering projects during cleanup.
///
/// These options control which projects are considered for cleaning based on
/// size and modification time criteria.
#[derive(Parser)]
struct FilteringArgs {
    /// Ignore projects with a build dir size smaller than the specified value
    ///
    /// Supports various size formats:
    /// - Decimal: KB, MB, GB (base 1000)
    /// - Binary: KiB, MiB, GiB (base 1024)
    /// - Bytes: plain numbers
    /// - Decimal values: 1.5MB, 2.5GiB, etc.
    #[arg(short = 's', long, default_value = "0")]
    keep_size: String,

    /// Ignore projects that have been compiled in the last \[DAYS\] days
    ///
    /// Projects with build directories modified within this timeframe will be
    /// skipped during cleanup. A value of 0 disables time-based filtering.
    #[arg(short = 'd', long, default_value = "0")]
    keep_days: u32,
}

/// Command-line arguments for controlling cleanup execution behavior.
///
/// These options determine how the cleanup process runs, including confirmation
/// prompts, dry-run mode, and interactive selection.
#[derive(Parser)]
struct ExecutionArgs {
    /// Don't ask for confirmation; Just clean all detected projects
    ///
    /// When enabled, it automatically proceeds with cleaning without any user prompts.
    /// Use with caution as this will immediately delete build directories.
    #[arg(short = 'y', long)]
    yes: bool,

    /// Collect the cleanable projects and list the reclaimable space
    ///
    /// When enabled, performs all scans and filtering but doesn't
    /// delete any files. Useful for previewing what would be cleaned.
    #[arg(long)]
    dry_run: bool,

    /// Use interactive project selection
    ///
    /// When enabled, it presents a list of found projects and allows the user to
    /// select which ones to clean using an interactive interface.
    #[arg(short = 'i', long)]
    interactive: bool,
}

/// Command-line arguments for filtering projects by type.
///
/// These options restrict cleaning to specific project types. The arguments
/// are mutually exclusive to prevent conflicting selections.
#[derive(Parser)]
#[allow(clippy::struct_excessive_bools)] // This is acceptable here due to the nature of the CLI
struct ProjectTypeArgs {
    /// Clean only Rust projects
    ///
    /// When enabled, only directories containing `Cargo.toml` and `target/`
    /// will be considered for cleanup.
    #[arg(long, conflicts_with_all = ["node_only", "python_only", "go_only"])]
    rust_only: bool,

    /// Clean only Node.js projects
    ///
    /// When enabled, only directories containing `package.json` and `node_modules/`
    /// will be considered for cleanup.
    #[arg(long, conflicts_with_all = ["rust_only", "python_only", "go_only"])]
    node_only: bool,

    /// Clean only Python projects
    ///
    /// When enabled, only directories containing Python configuration files
    /// (requirements.txt, setup.py, pyproject.toml) and cache directories
    /// (`__pycache__`, `.pytest_cache`, venv, .venv) will be considered for cleanup.
    #[arg(long, conflicts_with_all = ["rust_only", "node_only", "go_only"])]
    python_only: bool,

    /// Clean only Go projects
    ///
    /// When enabled, only directories containing `go.mod` and `vendor/`
    /// will be considered for cleanup.
    #[arg(long, conflicts_with_all = ["rust_only", "node_only", "python_only"])]
    go_only: bool,
}

/// Command-line arguments for controlling directory scanning behavior.
///
/// These options affect how directories are traversed and what information
/// is collected during the scanning phase.
#[derive(Parser)]
struct ScanningArgs {
    /// The number of threads to use for directory scanning
    ///
    /// A value of 0 uses the default number of threads (typically the number of CPU cores).
    /// Higher values can improve scanning performance on systems with fast storage.
    #[arg(short = 't', long, default_value = "0")]
    threads: usize,

    /// Show access errors that occur while scanning
    ///
    /// When enabled, displays errors encountered while accessing files or directories
    /// during the scanning process. Useful for debugging permission issues.
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Directories to ignore by default
    ///
    /// These directories will be completely ignored during scanning. Can be specified
    /// multiple times to ignore multiple directory patterns.
    #[arg(long, action = clap::ArgAction::Append)]
    ignore: Vec<PathBuf>,

    /// Directories to skip during scanning
    ///
    /// These directories will be skipped during scans, but their parent directories
    /// may still be processed. Can be specified multiple times.
    #[arg(long, action = clap::ArgAction::Append)]
    skip: Vec<PathBuf>,
}

/// Main command-line interface structure.
///
/// This struct defines the complete command-line interface for the clean-dev-dirs tool,
/// combining all argument groups and providing the main entry point for command parsing.
#[derive(Parser)]
#[command(name = "clean-dev-dirs")]
#[command(about = "Recursively clean Rust, Node.js, Python, and Go development directories")]
pub(crate) struct Cli {
    /// The directory to search for projects
    ///
    /// Specifies the root directory where the tool will recursively search for
    /// development projects. Defaults to the current directory if not specified.
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

/// Configuration for cleanup execution behavior.
///
/// This struct provides a simplified interface to execution-related options,
/// extracted from the command-line arguments.
#[derive(Clone)]
pub(crate) struct ExecutionOptions {
    /// Whether to run in dry-run mode (no actual deletion)
    pub(crate) dry_run: bool,

    /// Whether to use interactive project selection
    pub(crate) interactive: bool,
}

/// Configuration for project filtering criteria.
///
/// This struct contains the filtering options used to determine which projects
/// should be considered for cleanup based on size and modification time.
#[derive(Clone)]
pub struct FilterOptions {
    /// Minimum size threshold for build directories
    pub keep_size: String,

    /// Minimum age in days for projects to be considered
    pub keep_days: u32,
}

/// Enumeration of supported project type filters.
///
/// This enum is used to restrict scanning and cleaning to specific types of
/// development projects.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ProjectFilter {
    /// Include all supported project types (Rust, Node.js, Python, Go)
    All,

    /// Include only Rust projects (Cargo.toml + target/)
    RustOnly,

    /// Include only Node.js projects (package.json + `node_modules`/)
    NodeOnly,

    /// Include only Python projects (requirements.txt/setup.py/pyproject.toml + cache dirs)
    PythonOnly,

    /// Include only Go projects (go.mod + vendor/)
    GoOnly,
}

/// Configuration for directory scanning behavior.
///
/// This struct contains options that control how the directory tree is traversed
/// and what information is collected during scanning.
#[derive(Clone)]
pub struct ScanOptions {
    /// Whether to show verbose output including errors
    pub verbose: bool,

    /// Number of threads to use for parallel scanning
    pub threads: usize,

    /// Directories to skip during scanning
    pub skip: Vec<PathBuf>,
}

impl Cli {
    /// Extract project filter from command-line arguments.
    ///
    /// Converts the mutually exclusive project type flags into a single
    /// [`ProjectFilter`] enum value.
    ///
    /// # Returns
    ///
    /// - [`ProjectFilter::RustOnly`] if `--rust-only` was specified
    /// - [`ProjectFilter::NodeOnly`] if `--node-only` was specified  
    /// - [`ProjectFilter::PythonOnly`] if `--python-only` was specified
    /// - [`ProjectFilter::GoOnly`] if `--go-only` was specified
    /// - [`ProjectFilter::All`] if no specific flag was specified
    pub(crate) fn project_filter(&self) -> ProjectFilter {
        if self.project_type.rust_only {
            ProjectFilter::RustOnly
        } else if self.project_type.node_only {
            ProjectFilter::NodeOnly
        } else if self.project_type.python_only {
            ProjectFilter::PythonOnly
        } else if self.project_type.go_only {
            ProjectFilter::GoOnly
        } else {
            ProjectFilter::All
        }
    }

    /// Extract execution options from command-line arguments.
    ///
    /// Creates an [`ExecutionOptions`] struct containing the execution-related
    /// configuration from the parsed command-line arguments.
    ///
    /// # Returns
    ///
    /// An [`ExecutionOptions`] struct with the current execution configuration.
    pub(crate) fn execution_options(&self) -> ExecutionOptions {
        ExecutionOptions {
            dry_run: self.execution.dry_run,
            interactive: self.execution.interactive,
        }
    }

    /// Extract filtering options from command-line arguments.
    ///
    /// Creates a [`FilterOptions`] struct containing the filtering criteria
    /// from the parsed command-line arguments.
    ///
    /// # Returns
    ///
    /// A [`FilterOptions`] struct with the current filtering configuration.
    pub(crate) fn filter_options(&self) -> FilterOptions {
        FilterOptions {
            keep_size: self.filtering.keep_size.clone(),
            keep_days: self.filtering.keep_days,
        }
    }

    /// Extract scanning options from command-line arguments.
    ///
    /// Creates a [`ScanOptions`] struct containing the scanning-related
    /// configuration from the parsed command-line arguments.
    ///
    /// # Returns
    ///
    /// A [`ScanOptions`] struct with the current scanning configuration.
    pub(crate) fn scan_options(&self) -> ScanOptions {
        ScanOptions {
            verbose: self.scanning.verbose,
            threads: self.scanning.threads,
            skip: self.scanning.skip.clone(),
        }
    }
}
