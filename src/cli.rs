//! Command-line interface definition and argument parsing.
//!
//! This module defines all command-line arguments, options, and their validation
//! using the [clap](https://docs.rs/clap/) library. It provides structured access
//! to user input and handles argument conflicts and defaults.
//!
//! Helper methods on [`Cli`] accept a [`FileConfig`] reference so that config-file
//! values act as defaults that CLI arguments can override (layered config).

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use clean_dev_dirs::config::file::{FileConfig, expand_tilde};
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
    #[arg(short = 's', long)]
    keep_size: Option<String>,

    /// Ignore projects that have been compiled in the last \[DAYS\] days
    ///
    /// Projects with build directories modified within this timeframe will be
    /// skipped during cleanup. A value of 0 disables time-based filtering.
    #[arg(short = 'd', long)]
    keep_days: Option<u32>,
}

/// Command-line arguments for controlling cleanup execution behavior.
///
/// These options determine how the cleanup process runs, including confirmation
/// prompts, dry-run mode, and interactive selection.
#[derive(Parser)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Copy compiled executables to <project>/bin/ before cleaning
    ///
    /// When enabled, preserves compiled binaries (e.g. from target/release/
    /// and target/debug/ for Rust projects) by copying them to a bin/ directory
    /// in the project root before deleting build directories.
    #[arg(short = 'k', long)]
    keep_executables: bool,
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
    #[arg(short = 't', long)]
    threads: Option<usize>,

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
///
/// Helper methods accept a [`FileConfig`] reference so that config-file values act as
/// defaults when the corresponding CLI argument is not provided.
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
    #[arg()]
    dir: Option<PathBuf>,

    /// Project type to clean (all, rust, node, python, go)
    ///
    /// Restricts cleaning to specific project types. If not specified, all
    /// supported project types will be considered.
    #[arg(short = 'p', long)]
    project_type: Option<ProjectFilter>,

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
    /// Resolve the target directory from CLI args, config file, or default.
    ///
    /// Priority: CLI argument > config file > current directory (`.`).
    /// Tilde expansion is applied to paths originating from the config file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::config::FileConfig;
    /// # use std::path::PathBuf;
    /// # mod cli { include!("cli.rs"); }
    /// # use cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "/custom/path"]);
    /// assert_eq!(args.directory(&FileConfig::default()), PathBuf::from("/custom/path"));
    /// ```
    #[must_use]
    pub fn directory(&self, config: &FileConfig) -> PathBuf {
        if let Some(ref dir) = self.dir {
            return dir.clone();
        }

        if let Some(ref dir) = config.dir {
            return expand_tilde(dir);
        }

        PathBuf::from(".")
    }

    /// Extract project filter from CLI args and config file.
    ///
    /// Priority: CLI argument > config file > default (`All`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::config::{FileConfig, ProjectFilter};
    /// # mod cli { include!("cli.rs"); }
    /// # use cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--project-type", "rust"]);
    /// assert_eq!(args.project_filter(&FileConfig::default()), ProjectFilter::Rust);
    /// ```
    #[must_use]
    pub fn project_filter(&self, config: &FileConfig) -> ProjectFilter {
        self.project_type
            .or_else(|| {
                config
                    .project_type
                    .as_ref()
                    .and_then(|s| ProjectFilter::from_str(s, true).ok())
            })
            .unwrap_or_default()
    }

    /// Extract execution options from CLI args and config file.
    ///
    /// For boolean flags, the CLI flag (if set to `true`) takes priority,
    /// then the config file value, then `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::config::FileConfig;
    /// # mod cli { include!("cli.rs"); }
    /// # use cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--dry-run", "--interactive"]);
    /// let options = args.execution_options(&FileConfig::default());
    /// assert!(options.dry_run);
    /// assert!(options.interactive);
    /// ```
    #[must_use]
    pub fn execution_options(&self, config: &FileConfig) -> ExecutionOptions {
        ExecutionOptions {
            dry_run: self.execution.dry_run || config.execution.dry_run.unwrap_or(false),
            interactive: self.execution.interactive
                || config.execution.interactive.unwrap_or(false),
            keep_executables: self.execution.keep_executables
                || config.execution.keep_executables.unwrap_or(false),
        }
    }

    /// Extract scanning options from CLI args and config file.
    ///
    /// - **threads**: CLI > config > `0` (default)
    /// - **verbose**: CLI flag `||` config value `||` `false`
    /// - **skip**: merged from both sources (config values first, then CLI)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::config::FileConfig;
    /// # mod cli { include!("cli.rs"); }
    /// # use cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--verbose", "--threads", "4"]);
    /// let options = args.scan_options(&FileConfig::default());
    /// assert!(options.verbose);
    /// assert_eq!(options.threads, 4);
    /// ```
    #[must_use]
    pub fn scan_options(&self, config: &FileConfig) -> ScanOptions {
        let mut skip = config.scanning.skip.clone().unwrap_or_default();
        skip.extend(self.scanning.skip.clone());

        ScanOptions {
            verbose: self.scanning.verbose || config.scanning.verbose.unwrap_or(false),
            threads: self
                .scanning
                .threads
                .or(config.scanning.threads)
                .unwrap_or(0),
            skip,
        }
    }

    /// Extract filtering options from CLI args and config file.
    ///
    /// Priority: CLI argument > config file > hardcoded default.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clap::Parser;
    /// # use clean_dev_dirs::config::FileConfig;
    /// # mod cli { include!("cli.rs"); }
    /// # use cli::Cli;
    /// let args = Cli::parse_from(&["clean-dev-dirs", "--keep-size", "100MB", "--keep-days", "30"]);
    /// let options = args.filter_options(&FileConfig::default());
    /// assert_eq!(options.keep_size, "100MB");
    /// assert_eq!(options.keep_days, 30);
    /// ```
    #[must_use]
    pub fn filter_options(&self, config: &FileConfig) -> FilterOptions {
        FilterOptions {
            keep_size: self
                .filtering
                .keep_size
                .clone()
                .or_else(|| config.filtering.keep_size.clone())
                .unwrap_or_else(|| "0".to_string()),
            keep_days: self
                .filtering
                .keep_days
                .or(config.filtering.keep_days)
                .unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use clean_dev_dirs::config::file::{
        FileConfig, FileExecutionConfig, FileFilterConfig, FileScanConfig,
    };

    // ── Existing tests (updated for FileConfig parameter) ──────────────

    #[test]
    fn test_default_values() {
        let args = Cli::parse_from(["clean-dev-dirs"]);
        let config = FileConfig::default();

        assert_eq!(args.directory(&config), PathBuf::from("."));
        assert_eq!(args.project_filter(&config), ProjectFilter::All);

        let exec_opts = args.execution_options(&config);
        assert!(!exec_opts.dry_run);
        assert!(!exec_opts.interactive);
        assert!(!exec_opts.keep_executables);

        let scan_opts = args.scan_options(&config);
        assert!(!scan_opts.verbose);
        assert_eq!(scan_opts.threads, 0);
        assert!(scan_opts.skip.is_empty());

        let filter_opts = args.filter_options(&config);
        assert_eq!(filter_opts.keep_size, "0");
        assert_eq!(filter_opts.keep_days, 0);
    }

    #[test]
    fn test_project_filters() {
        let config = FileConfig::default();

        let rust_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "rust"]);
        assert_eq!(rust_args.project_filter(&config), ProjectFilter::Rust);

        let node_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "node"]);
        assert_eq!(node_args.project_filter(&config), ProjectFilter::Node);

        let python_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "python"]);
        assert_eq!(python_args.project_filter(&config), ProjectFilter::Python);

        let go_args = Cli::parse_from(["clean-dev-dirs", "--project-type", "go"]);
        assert_eq!(go_args.project_filter(&config), ProjectFilter::Go);

        let all_args = Cli::parse_from(["clean-dev-dirs"]);
        assert_eq!(all_args.project_filter(&config), ProjectFilter::All);
    }

    #[test]
    fn test_project_filter_short_flag() {
        let config = FileConfig::default();
        let rust_args = Cli::parse_from(["clean-dev-dirs", "-p", "rust"]);
        assert_eq!(rust_args.project_filter(&config), ProjectFilter::Rust);
    }

    #[test]
    fn test_execution_options() {
        let config = FileConfig::default();
        let args = Cli::parse_from(["clean-dev-dirs", "--dry-run", "--interactive", "--yes"]);
        let exec_opts = args.execution_options(&config);

        assert!(exec_opts.dry_run);
        assert!(exec_opts.interactive);
        assert!(!exec_opts.keep_executables);
    }

    #[test]
    fn test_keep_executables_flag() {
        let config = FileConfig::default();

        let args = Cli::parse_from(["clean-dev-dirs", "--keep-executables"]);
        let exec_opts = args.execution_options(&config);
        assert!(exec_opts.keep_executables);

        let args_short = Cli::parse_from(["clean-dev-dirs", "-k"]);
        let exec_opts_short = args_short.execution_options(&config);
        assert!(exec_opts_short.keep_executables);
    }

    #[test]
    fn test_scanning_options() {
        let config = FileConfig::default();
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
        let scan_opts = args.scan_options(&config);

        assert!(scan_opts.verbose);
        assert_eq!(scan_opts.threads, 8);
        assert_eq!(scan_opts.skip.len(), 2);
        assert!(scan_opts.skip.contains(&PathBuf::from("node_modules")));
        assert!(scan_opts.skip.contains(&PathBuf::from(".git")));
    }

    #[test]
    fn test_filtering_options() {
        let config = FileConfig::default();
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--keep-size",
            "100MB",
            "--keep-days",
            "30",
        ]);
        let filter_opts = args.filter_options(&config);

        assert_eq!(filter_opts.keep_size, "100MB");
        assert_eq!(filter_opts.keep_days, 30);
    }

    #[test]
    fn test_custom_directory() {
        let config = FileConfig::default();
        let args = Cli::parse_from(["clean-dev-dirs", "/custom/path"]);
        assert_eq!(args.directory(&config), PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_short_flags() {
        let config = FileConfig::default();
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

        let filter_opts = args.filter_options(&config);
        assert_eq!(filter_opts.keep_size, "50MB");
        assert_eq!(filter_opts.keep_days, 7);

        let scan_opts = args.scan_options(&config);
        assert_eq!(scan_opts.threads, 2);
        assert!(scan_opts.verbose);

        let exec_opts = args.execution_options(&config);
        assert!(exec_opts.interactive);
    }

    #[test]
    fn test_multiple_skip_directories() {
        let config = FileConfig::default();
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

        let scan_opts = args.scan_options(&config);
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
        let config = FileConfig::default();
        let test_cases = vec![
            ("100KB", "100KB"),
            ("1.5MB", "1.5MB"),
            ("2GiB", "2GiB"),
            ("500000", "500000"),
        ];

        for (input, expected) in test_cases {
            let args = Cli::parse_from(["clean-dev-dirs", "--keep-size", input]);
            let filter_opts = args.filter_options(&config);
            assert_eq!(filter_opts.keep_size, expected);
        }
    }

    #[test]
    fn test_zero_values() {
        let config = FileConfig::default();
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "--keep-size",
            "0",
            "--keep-days",
            "0",
            "--threads",
            "0",
        ]);

        let filter_opts = args.filter_options(&config);
        assert_eq!(filter_opts.keep_size, "0");
        assert_eq!(filter_opts.keep_days, 0);

        let scan_opts = args.scan_options(&config);
        assert_eq!(scan_opts.threads, 0);
    }

    // ── Config merging tests ───────────────────────────────────────────

    #[test]
    fn test_config_values_used_when_cli_absent() {
        let args = Cli::parse_from(["clean-dev-dirs"]);
        let config = FileConfig {
            project_type: Some("rust".to_string()),
            dir: Some(PathBuf::from("/config/dir")),
            filtering: FileFilterConfig {
                keep_size: Some("50MB".to_string()),
                keep_days: Some(7),
            },
            scanning: FileScanConfig {
                threads: Some(4),
                verbose: Some(true),
                skip: Some(vec![PathBuf::from(".cargo")]),
                ignore: Some(vec![PathBuf::from(".git")]),
            },
            execution: FileExecutionConfig {
                keep_executables: Some(true),
                interactive: Some(true),
                dry_run: Some(true),
            },
        };

        assert_eq!(args.directory(&config), PathBuf::from("/config/dir"));
        assert_eq!(args.project_filter(&config), ProjectFilter::Rust);

        let filter_opts = args.filter_options(&config);
        assert_eq!(filter_opts.keep_size, "50MB");
        assert_eq!(filter_opts.keep_days, 7);

        let scan_opts = args.scan_options(&config);
        assert_eq!(scan_opts.threads, 4);
        assert!(scan_opts.verbose);
        assert_eq!(scan_opts.skip, vec![PathBuf::from(".cargo")]);

        let exec_opts = args.execution_options(&config);
        assert!(exec_opts.keep_executables);
        assert!(exec_opts.interactive);
        assert!(exec_opts.dry_run);
    }

    #[test]
    fn test_cli_overrides_config_values() {
        let args = Cli::parse_from([
            "clean-dev-dirs",
            "/cli/dir",
            "--project-type",
            "node",
            "--keep-size",
            "100MB",
            "--keep-days",
            "30",
            "--threads",
            "8",
        ]);
        let config = FileConfig {
            project_type: Some("rust".to_string()),
            dir: Some(PathBuf::from("/config/dir")),
            filtering: FileFilterConfig {
                keep_size: Some("50MB".to_string()),
                keep_days: Some(7),
            },
            scanning: FileScanConfig {
                threads: Some(4),
                ..FileScanConfig::default()
            },
            ..FileConfig::default()
        };

        assert_eq!(args.directory(&config), PathBuf::from("/cli/dir"));
        assert_eq!(args.project_filter(&config), ProjectFilter::Node);

        let filter_opts = args.filter_options(&config);
        assert_eq!(filter_opts.keep_size, "100MB");
        assert_eq!(filter_opts.keep_days, 30);

        let scan_opts = args.scan_options(&config);
        assert_eq!(scan_opts.threads, 8);
    }

    #[test]
    fn test_skip_dirs_merged_from_both_sources() {
        let args = Cli::parse_from(["clean-dev-dirs", "--skip", "node_modules"]);
        let config = FileConfig {
            scanning: FileScanConfig {
                skip: Some(vec![PathBuf::from(".cargo"), PathBuf::from("vendor")]),
                ..FileScanConfig::default()
            },
            ..FileConfig::default()
        };

        let scan_opts = args.scan_options(&config);
        assert_eq!(scan_opts.skip.len(), 3);
        assert!(scan_opts.skip.contains(&PathBuf::from(".cargo")));
        assert!(scan_opts.skip.contains(&PathBuf::from("vendor")));
        assert!(scan_opts.skip.contains(&PathBuf::from("node_modules")));
    }

    #[test]
    fn test_bool_flags_override_config_false() {
        let args = Cli::parse_from(["clean-dev-dirs", "--dry-run"]);
        let config = FileConfig {
            execution: FileExecutionConfig {
                dry_run: Some(false),
                interactive: Some(true),
                keep_executables: Some(false),
            },
            ..FileConfig::default()
        };

        let exec_opts = args.execution_options(&config);
        assert!(exec_opts.dry_run);
        assert!(exec_opts.interactive);
        assert!(!exec_opts.keep_executables);
    }

    #[test]
    fn test_config_dir_with_tilde_expansion() {
        let args = Cli::parse_from(["clean-dev-dirs"]);
        let config = FileConfig {
            dir: Some(PathBuf::from("~/Projects")),
            ..FileConfig::default()
        };

        let dir = args.directory(&config);
        if let Some(home) = dirs::home_dir() {
            assert_eq!(dir, home.join("Projects"));
        }
    }

    #[test]
    fn test_config_project_type_case_insensitive() {
        let args = Cli::parse_from(["clean-dev-dirs"]);

        let config_upper = FileConfig {
            project_type: Some("Rust".to_string()),
            ..FileConfig::default()
        };
        assert_eq!(args.project_filter(&config_upper), ProjectFilter::Rust);

        let config_mixed = FileConfig {
            project_type: Some("NODE".to_string()),
            ..FileConfig::default()
        };
        assert_eq!(args.project_filter(&config_mixed), ProjectFilter::Node);
    }

    #[test]
    fn test_invalid_config_project_type_falls_back_to_default() {
        let args = Cli::parse_from(["clean-dev-dirs"]);
        let config = FileConfig {
            project_type: Some("invalid_type".to_string()),
            ..FileConfig::default()
        };

        assert_eq!(args.project_filter(&config), ProjectFilter::All);
    }
}
