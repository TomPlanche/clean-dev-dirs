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
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
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
#[allow(dead_code)]
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ProjectFilter {
    /// Include all supported project types (Rust, Node.js, Python, Go)
    All,

    /// Include only Rust projects (Cargo.toml + target/)
    RustOnly,

    /// Include only Node.js projects (package.json + `node_modules`/)
    NodeOnly,

    /// Include only Python projects (Python config files + cache dirs)
    PythonOnly,

    /// Include only Go projects (go.mod + vendor/)
    GoOnly,
}

/// Configuration for directory scanning behavior.
///
/// This struct contains options that control how directories are traversed
/// and what information is collected during the scanning process.
#[derive(Clone)]
pub struct ScanOptions {
    /// Whether to show verbose output including scan errors
    pub verbose: bool,

    /// Number of threads to use for scanning (0 = default)
    pub threads: usize,

    /// List of directory patterns to skip during scanning
    pub skip: Vec<PathBuf>,
}

impl Cli {
    /// Extract project filter from command-line arguments.
    ///
    /// This method analyzes the project type flags and returns the appropriate
    /// filter enum value. Only one project type can be selected at a time due
    /// to the `conflicts_with_all` constraints in the argument definitions.
    ///
    /// # Returns
    ///
    /// - `ProjectFilter::RustOnly` if `--rust-only` is specified
    /// - `ProjectFilter::NodeOnly` if `--node-only` is specified
    /// - `ProjectFilter::PythonOnly` if `--python-only` is specified
    /// - `ProjectFilter::GoOnly` if `--go-only` is specified
    /// - `ProjectFilter::All` if no specific project type is specified
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use crate::cli::{Cli, ProjectFilter};
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--rust-only"]);
    /// assert_eq!(args.project_filter(), ProjectFilter::RustOnly);
    /// ```
    #[allow(dead_code)]
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
    /// This method creates an `ExecutionOptions` struct containing the
    /// execution-related settings specified by the user.
    ///
    /// # Returns
    ///
    /// An `ExecutionOptions` struct with the dry-run and interactive flags
    /// extracted from the command-line arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use crate::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--dry-run", "--interactive"]);
    /// let options = args.execution_options();
    /// assert!(options.dry_run);
    /// assert!(options.interactive);
    /// ```
    #[allow(dead_code)]
    pub(crate) fn execution_options(&self) -> ExecutionOptions {
        ExecutionOptions {
            dry_run: self.execution.dry_run,
            interactive: self.execution.interactive,
        }
    }

    /// Extract scanning options from command-line arguments.
    ///
    /// This method creates a `ScanOptions` struct containing the
    /// scanning-related settings specified by the user.
    ///
    /// # Returns
    ///
    /// A `ScanOptions` struct with verbose, threads, and skip options
    /// extracted from the command-line arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use crate::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--verbose", "--threads", "4"]);
    /// let options = args.scan_options();
    /// assert!(options.verbose);
    /// assert_eq!(options.threads, 4);
    /// ```
    #[allow(dead_code)]
    pub(crate) fn scan_options(&self) -> ScanOptions {
        ScanOptions {
            verbose: self.scanning.verbose,
            threads: self.scanning.threads,
            skip: self.scanning.skip.clone(),
        }
    }

    /// Extract filtering options from command-line arguments.
    ///
    /// This method creates a `FilterOptions` struct containing the
    /// filtering criteria specified by the user.
    ///
    /// # Returns
    ///
    /// A `FilterOptions` struct with size and time filtering criteria
    /// extracted from the command-line arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use crate::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--keep-size", "100MB", "--keep-days", "30"]);
    /// let options = args.filter_options();
    /// assert_eq!(options.keep_size, "100MB");
    /// assert_eq!(options.keep_days, 30);
    /// ```
    #[allow(dead_code)]
    pub(crate) fn filter_options(&self) -> FilterOptions {
        FilterOptions {
            keep_size: self.filtering.keep_size.clone(),
            keep_days: self.filtering.keep_days,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_default_values() {
        let args = Cli::parse_from(["clean-dev-dirs"]);

        assert_eq!(args.dir, PathBuf::from("."));
        assert_eq!(args.project_filter(), ProjectFilter::All);

        let exec_opts = args.execution_options();
        assert!(!exec_opts.dry_run);
        assert!(!exec_opts.interactive);

        let scan_opts = args.scan_options();
        assert!(!scan_opts.verbose);
        assert_eq!(scan_opts.threads, 0);
        assert!(scan_opts.skip.is_empty());

        let filter_opts = args.filter_options();
        assert_eq!(filter_opts.keep_size, "0");
        assert_eq!(filter_opts.keep_days, 0);
    }

    #[test]
    fn test_project_filters() {
        let rust_args = Cli::parse_from(["clean-dev-dirs", "--rust-only"]);
        assert_eq!(rust_args.project_filter(), ProjectFilter::RustOnly);

        let node_args = Cli::parse_from(["clean-dev-dirs", "--node-only"]);
        assert_eq!(node_args.project_filter(), ProjectFilter::NodeOnly);

        let python_args = Cli::parse_from(["clean-dev-dirs", "--python-only"]);
        assert_eq!(python_args.project_filter(), ProjectFilter::PythonOnly);

        let go_args = Cli::parse_from(["clean-dev-dirs", "--go-only"]);
        assert_eq!(go_args.project_filter(), ProjectFilter::GoOnly);

        let all_args = Cli::parse_from(["clean-dev-dirs"]);
        assert_eq!(all_args.project_filter(), ProjectFilter::All);
    }

    #[test]
    fn test_execution_options() {
        let args = Cli::parse_from(["clean-dev-dirs", "--dry-run", "--interactive", "--yes"]);
        let exec_opts = args.execution_options();

        assert!(exec_opts.dry_run);
        assert!(exec_opts.interactive);
    }

    #[test]
    fn test_scanning_options() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--verbose",
            "--threads",
            "8",
            "--skip",
            "node_modules",
            "--skip",
            ".git",
        ]);
        let scan_opts = args.scan_options();

        assert!(scan_opts.verbose);
        assert_eq!(scan_opts.threads, 8);
        assert_eq!(scan_opts.skip.len(), 2);
        assert!(scan_opts.skip.contains(&PathBuf::from("node_modules")));
        assert!(scan_opts.skip.contains(&PathBuf::from(".git")));
    }

    #[test]
    fn test_filtering_options() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--keep-size",
            "100MB",
            "--keep-days",
            "30",
        ]);
        let filter_opts = args.filter_options();

        assert_eq!(filter_opts.keep_size, "100MB");
        assert_eq!(filter_opts.keep_days, 30);
    }

    #[test]
    fn test_custom_directory() {
        let args = Cli::parse_from(["clean-dev-dirs", "/custom/path"]);
        assert_eq!(args.dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_project_filter_equality() {
        assert_eq!(ProjectFilter::All, ProjectFilter::All);
        assert_eq!(ProjectFilter::RustOnly, ProjectFilter::RustOnly);
        assert_eq!(ProjectFilter::NodeOnly, ProjectFilter::NodeOnly);
        assert_eq!(ProjectFilter::PythonOnly, ProjectFilter::PythonOnly);
        assert_eq!(ProjectFilter::GoOnly, ProjectFilter::GoOnly);

        assert_ne!(ProjectFilter::All, ProjectFilter::RustOnly);
        assert_ne!(ProjectFilter::RustOnly, ProjectFilter::NodeOnly);
        assert_ne!(ProjectFilter::NodeOnly, ProjectFilter::PythonOnly);
        assert_ne!(ProjectFilter::PythonOnly, ProjectFilter::GoOnly);
    }

    #[test]
    fn test_execution_options_clone() {
        let original = ExecutionOptions {
            dry_run: true,
            interactive: false,
        };
        let cloned = original.clone();

        assert_eq!(original.dry_run, cloned.dry_run);
        assert_eq!(original.interactive, cloned.interactive);
    }

    #[test]
    fn test_filter_options_clone() {
        let original = FilterOptions {
            keep_size: "100MB".to_string(),
            keep_days: 30,
        };
        let cloned = original.clone();

        assert_eq!(original.keep_size, cloned.keep_size);
        assert_eq!(original.keep_days, cloned.keep_days);
    }

    #[test]
    fn test_scan_options_clone() {
        let original = ScanOptions {
            verbose: true,
            threads: 4,
            skip: vec![PathBuf::from("test")],
        };
        let cloned = original.clone();

        assert_eq!(original.verbose, cloned.verbose);
        assert_eq!(original.threads, cloned.threads);
        assert_eq!(original.skip, cloned.skip);
    }

    #[test]
    fn test_project_filter_copy() {
        let original = ProjectFilter::RustOnly;
        let copied = original;

        assert_eq!(original, copied);
    }

    #[test]
    fn test_short_flags() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "-s",
            "50MB",
            "-d",
            "7",
            "-t",
            "2",
            "-v",
            "-i",
            "-y",
        ]);

        let filter_opts = args.filter_options();
        assert_eq!(filter_opts.keep_size, "50MB");
        assert_eq!(filter_opts.keep_days, 7);

        let scan_opts = args.scan_options();
        assert_eq!(scan_opts.threads, 2);
        assert!(scan_opts.verbose);

        let exec_opts = args.execution_options();
        assert!(exec_opts.interactive);
    }

    #[test]
    fn test_multiple_skip_directories() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--skip",
            "node_modules",
            "--skip",
            ".git",
            "--skip",
            "target",
            "--skip",
            "__pycache__",
        ]);

        let scan_opts = args.scan_options();
        assert_eq!(scan_opts.skip.len(), 4);

        let expected_dirs = vec![
            PathBuf::from("node_modules"),
            PathBuf::from(".git"),
            PathBuf::from("target"),
            PathBuf::from("__pycache__"),
        ];

        for expected_dir in expected_dirs {
            assert!(scan_opts.skip.contains(&expected_dir));
        }
    }

    #[test]
    fn test_complex_size_formats() {
        let test_cases = vec![
            ("100KB", "100KB"),
            ("1.5MB", "1.5MB"),
            ("2GiB", "2GiB"),
            ("500000", "500000"),
        ];

        for (input, expected) in test_cases {
            let args = Cli::parse_from(["clean-dev-dirs", "--keep-size", input]);
            let filter_opts = args.filter_options();
            assert_eq!(filter_opts.keep_size, expected);
        }
    }

    #[test]
    fn test_zero_values() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--keep-size",
            "0",
            "--keep-days",
            "0",
            "--threads",
            "0",
        ]);

        let filter_opts = args.filter_options();
        assert_eq!(filter_opts.keep_size, "0");
        assert_eq!(filter_opts.keep_days, 0);

        let scan_opts = args.scan_options();
        assert_eq!(scan_opts.threads, 0);
    }
}
