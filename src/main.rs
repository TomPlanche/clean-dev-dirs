//! # clean-dev-dirs
//!
//! A fast and efficient CLI tool for recursively cleaning Rust `target/` and Node.js `node_modules/`
//! directories to reclaim disk space.
//!
//! This tool scans directories to find development projects and their build artifacts, allowing
//! you to selectively or automatically clean them to free up disk space. It supports parallel
//! scanning, intelligent filtering, and interactive selection.
//!
//! ## Features
//!
//! - Multi-language support (Rust, Node.js, Python, and Go)
//! - Parallel directory scanning
//! - Size and time-based filtering
//! - Interactive project selection
//! - Dry-run mode for safe previewing
//! - Human-readable output with progress indicators
//! - Persistent configuration via `~/.config/clean-dev-dirs/config.toml`
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage - clean current directory
//! clean-dev-dirs
//!
//! # Clean with filters
//! clean-dev-dirs --keep-size 100MB --keep-days 30
//!
//! # Interactive mode
//! clean-dev-dirs --interactive
//! ```

mod cli;

use anyhow::{Ok, Result, bail};
use clap::Parser;
use clean_dev_dirs::{
    cleaner::Cleaner, config::FileConfig, filtering::filter_projects, output::JsonOutput,
    project::Projects, scanner::Scanner,
};
use cli::Cli;
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use inquire::Confirm;
use std::process::exit;

/// Entry point for the clean-dev-dirs application.
///
/// This function handles all errors gracefully by calling [`inner_main`] and printing
/// any errors to stderr before exiting with a non-zero status code.
fn main() {
    if let Err(err) = inner_main() {
        eprintln!("Error: {err}");

        exit(1);
    }
}

/// Main application logic that can return errors.
///
/// This function:
/// 1. Parses command-line arguments
/// 2. Loads the persistent configuration file (if present)
/// 3. Configures the thread pool for parallel processing
/// 4. Scans the specified directory for development projects
/// 5. Filters projects based on user criteria
/// 6. Either performs a dry run, interactive selection, or automatic cleaning
/// 7. If `--json` is active, emits a single JSON document to stdout
///
/// # Returns
///
/// - `Ok(())` if the operation completed successfully
/// - `Err(anyhow::Error)` if any error occurred during execution
///
/// # Errors
///
/// This function can return errors from:
/// - Thread pool configuration
/// - Directory scanning
/// - Project filtering
/// - Interactive selection
/// - File system operations during cleaning
/// - JSON serialization
fn inner_main() -> Result<()> {
    let args = Cli::parse();
    let json_mode = args.json();

    let file_config = match FileConfig::load() {
        std::result::Result::Ok(config) => config,
        Err(e) => {
            if !json_mode {
                eprintln!("{} {e}", "Warning: Failed to load config file:".yellow());
            }
            FileConfig::default()
        }
    };

    let dir = args.directory(&file_config);

    let project_filter = args.project_filter(&file_config);
    let execution_options = args.execution_options(&file_config);
    let scan_options = args.scan_options(&file_config);
    let filter_options = args.filter_options(&file_config);

    // --json is incompatible with --interactive
    if json_mode && execution_options.interactive {
        bail!("--json and --interactive cannot be used together");
    }

    if scan_options.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(scan_options.threads)
            .build_global()?;
    }

    let scanner = Scanner::new(scan_options, project_filter).with_quiet(json_mode);

    let projects = scanner.scan_directory(&dir);

    if !json_mode {
        println!("Found {} projects", projects.len());
    }

    if projects.is_empty() {
        if json_mode {
            let output = JsonOutput::from_projects_dry_run(&[]);
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("{}", "✨ No development directories found!".green());
        }
        return Ok(());
    }

    let filtered_projects = filter_projects(projects, &filter_options)?;

    if filtered_projects.is_empty() {
        if json_mode {
            let output = JsonOutput::from_projects_dry_run(&[]);
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!(
                "{}",
                "✨ No directories match the specified criteria!".green()
            );
        }
        return Ok(());
    }

    let total_size: u64 = filtered_projects.iter().map(|p| p.build_arts.size).sum();

    if !json_mode {
        println!("\n{}", "📊 Found projects:".bold());
    }

    let projects: Projects = filtered_projects.into();

    if !json_mode {
        projects.print_summary(total_size);
    }

    let mut keep_executables = execution_options.keep_executables;

    if execution_options.interactive {
        let filtered_projects = projects.interactive_selection()?;

        if filtered_projects.is_empty() {
            println!("{}", "✨ No projects selected for cleaning!".green());

            return Ok(());
        }

        if !keep_executables {
            keep_executables = Confirm::new("Keep compiled executables before cleaning?")
                .with_default(false)
                .prompt()?;
        }
    }

    let final_size: u64 = projects.get_total_size();

    if execution_options.dry_run {
        if json_mode {
            let project_list = projects.as_slice();
            let output = JsonOutput::from_projects_dry_run(project_list);
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!(
                "\n{} {}",
                "🧪 Dry run complete!".yellow(),
                format!("Would free up {}", format_size(final_size, DECIMAL)).bright_white()
            );
        }
        return Ok(());
    }

    // Actual cleanup
    let project_snapshot: Vec<_> = projects.as_slice().to_vec();
    let clean_result = Cleaner::clean_projects(projects, keep_executables, json_mode);

    if json_mode {
        let output = JsonOutput::from_projects_cleanup(&project_snapshot, &clean_result);
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        Cleaner::print_summary(&clean_result);
    }

    Ok(())
}
