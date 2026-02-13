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

use anyhow::{Ok, Result};
use clap::Parser;
use clean_dev_dirs::{
    cleaner::Cleaner, filtering::filter_projects, project::Projects, scanner::Scanner,
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
/// 2. Configures the thread pool for parallel processing
/// 3. Scans the specified directory for development projects
/// 4. Filters projects based on user criteria
/// 5. Either performs a dry run, interactive selection, or automatic cleaning
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
fn inner_main() -> Result<()> {
    let args = Cli::parse();

    let dir = &args.dir;

    let project_filter = args.project_filter();
    let execution_options = args.execution_options();
    let scan_options = args.scan_options();
    let filter_options = args.filter_options();

    if scan_options.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(scan_options.threads)
            .build_global()?;
    }

    let scanner = Scanner::new(scan_options, project_filter);

    let projects = scanner.scan_directory(dir);

    println!("Found {} projects", projects.len());

    if projects.is_empty() {
        println!("{}", "✨ No development directories found!".green());
        return Ok(());
    }

    let filtered_projects = filter_projects(projects, &filter_options)?;

    if filtered_projects.is_empty() {
        println!(
            "{}",
            "✨ No directories match the specified criteria!".green()
        );
        return Ok(());
    }

    let total_size: u64 = filtered_projects.iter().map(|p| p.build_arts.size).sum();

    println!("\n{}", "📊 Found projects:".bold());

    let projects: Projects = filtered_projects.into();
    projects.print_summary(total_size);

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
        println!(
            "\n{} {}",
            "🧪 Dry run complete!".yellow(),
            format!("Would free up {}", format_size(final_size, DECIMAL)).bright_white()
        );
        return Ok(());
    }

    Cleaner::clean_projects(projects, keep_executables);

    Ok(())
}
