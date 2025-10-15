//! Command-line interface definition and argument parsing.
//!
//! This module defines all command-line arguments, options, and their validation
//! using the [clap](https://docs.rs/clap/) library. It provides structured access
//! to user input and handles argument conflicts and defaults.

use std::path::PathBuf;

use clap::Parser;

use clean_dev_dirs::config::{ExecutionOptions, FilterOptions, ProjectFilter, ScanOptions};

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
#[command(version)]
#[command(author)]
pub struct Cli {
    /// The directory to search for projects
    ///
    /// Specifies the root directory where the tool will recursively search for
    /// development projects. Defaults to the current directory if not specified.
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Project type to clean (all, rust, node, python, go)
    ///
    /// Restricts cleaning to specific project types. If not specified, all
    /// supported project types will be considered.
    #[arg(short = 'p', long, default_value = "all")]
    project_type: ProjectFilter,

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

impl Cli {
    /// Extract project filter from command-line arguments.
    ///
    /// This method returns the project type filter specified by the user.
    ///
    /// # Returns
    ///
    /// The `ProjectFilter` enum value from the CLI arguments.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::cli::Cli;
    /// # use clean_dev_dirs::config::ProjectFilter;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--project-type", "rust"]);
    /// assert_eq!(args.project_filter(), ProjectFilter::Rust);
    /// ```
    pub fn project_filter(&self) -> ProjectFilter {
        self.project_type
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
    /// # use clean_dev_dirs::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--dry-run", "--interactive"]);
    /// let options = args.execution_options();
    /// assert!(options.dry_run);
    /// assert!(options.interactive);
    /// ```
    pub fn execution_options(&self) -> ExecutionOptions {
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
    /// # use clean_dev_dirs::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--verbose", "--threads", "4"]);
    /// let options = args.scan_options();
    /// assert!(options.verbose);
    /// assert_eq!(options.threads, 4);
    /// ```
    pub fn scan_options(&self) -> ScanOptions {
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
    /// # use clean_dev_dirs::cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--keep-size", "100MB", "--keep-days", "30"]);
    /// let options = args.filter_options();
    /// assert_eq!(options.keep_size, "100MB");
    /// assert_eq!(options.keep_days, 30);
    /// ```
    pub fn filter_options(&self) -> FilterOptions {
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
        let rust_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "rust"]);
        assert_eq!(rust_args.project_filter(), ProjectFilter::Rust);

        let node_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "node"]);
        assert_eq!(node_args.project_filter(), ProjectFilter::Node);

        let python_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "python"]);
        assert_eq!(python_args.project_filter(), ProjectFilter::Python);

        let go_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "go"]);
        assert_eq!(go_args.project_filter(), ProjectFilter::Go);

        let all_args = Cli::parse_from(["clean-dev-dirs"]);
        assert_eq!(all_args.project_filter(), ProjectFilter::All);
    }

    #[test]
    fn test_project_filter_short_flag() {
        let rust_args = Cli::parse_from(["clean-dev-dirs", "-p", "rust"]);
        assert_eq!(rust_args.project_filter(), ProjectFilter::Rust);
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
