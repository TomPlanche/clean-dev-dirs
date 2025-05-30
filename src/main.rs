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
//! - Multi-language support (Rust and Node.js)
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

mod cleaner;
mod cli;
mod project;
mod scanner;

use anyhow::{Ok, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use cleaner::Cleaner;
use cli::{Cli, FilterOptions};
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use project::{Project, Projects};
use rayon::prelude::*;
use scanner::Scanner;
use std::{fs, process::exit};

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

    if execution_options.interactive {
        let filtered_projects = projects.interactive_selection()?;

        if filtered_projects.is_empty() {
            println!("{}", "✨ No projects selected for cleaning!".green());

            return Ok(());
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

    Cleaner::clean_projects(projects);

    Ok(())
}

/// Parse a human-readable size string into bytes.
///
/// Supports both decimal (KB, MB, GB) and binary (KiB, MiB, GiB) units,
/// as well as decimal numbers (e.g., "1.5GB").
///
/// # Arguments
///
/// * `size_str` - A string representing the size (e.g., "100MB", "1.5GiB", "1,000,000")
///
/// # Returns
///
/// - `Ok(u64)` - The size in bytes
/// - `Err(anyhow::Error)` - If the string format is invalid or causes overflow
///
/// # Examples
///
/// ```
/// # use anyhow::Result;
/// # fn parse_size(size_str: &str) -> Result<u64> { Ok(0) } // Mock for doc test
/// assert_eq!(parse_size("100KB")?, 100_000);
/// assert_eq!(parse_size("1.5MB")?, 1_500_000);
/// assert_eq!(parse_size("1GiB")?, 1_073_741_824);
/// ```
///
/// # Supported Units
///
/// - **Decimal**: KB (1000), MB (1000²), GB (1000³)
/// - **Binary**: KiB (1024), MiB (1024²), GiB (1024³)
/// - **Bytes**: Plain numbers without units
fn parse_size(size_str: &str) -> Result<u64> {
    if size_str == "0" {
        return Ok(0);
    }

    let size_str = size_str.to_uppercase();
    let (number_str, multiplier) = parse_size_unit(&size_str);

    if number_str.contains('.') {
        parse_decimal_size(number_str, multiplier)
    } else {
        parse_integer_size(number_str, multiplier)
    }
}

/// Parse the unit suffix and return the numeric part with its multiplier.
fn parse_size_unit(size_str: &str) -> (&str, u64) {
    const UNITS: &[(&str, u64)] = &[
        ("GIB", 1_073_741_824),
        ("MIB", 1_048_576),
        ("KIB", 1_024),
        ("GB", 1_000_000_000),
        ("MB", 1_000_000),
        ("KB", 1_000),
    ];

    for (suffix, multiplier) in UNITS {
        if size_str.ends_with(suffix) {
            return (size_str.trim_end_matches(suffix), *multiplier);
        }
    }

    (size_str, 1)
}

/// Parse a decimal size value (e.g., "1.5").
fn parse_decimal_size(number_str: &str, multiplier: u64) -> Result<u64> {
    let parts: Vec<&str> = number_str.split('.').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid decimal format: {number_str}"));
    }

    let integer_part: u64 = parts[0].parse().unwrap_or(0);
    let fractional_result = parse_fractional_part(parts[1])?;

    let integer_bytes = multiply_with_overflow_check(integer_part, multiplier)?;
    let fractional_bytes =
        multiply_with_overflow_check(fractional_result, multiplier)? / 1_000_000_000;

    add_with_overflow_check(integer_bytes, fractional_bytes)
}

/// Parse the fractional part of a decimal number.
fn parse_fractional_part(fractional_str: &str) -> Result<u64> {
    let fractional_digits = fractional_str.len();
    if fractional_digits > 9 {
        return Err(anyhow::anyhow!("Too many decimal places: {fractional_str}"));
    }

    let fractional_part: u64 = fractional_str.parse()?;
    let fractional_multiplier = 10u64.pow(9 - u32::try_from(fractional_digits)?);

    Ok(fractional_part * fractional_multiplier)
}

/// Parse an integer size value.
fn parse_integer_size(number_str: &str, multiplier: u64) -> Result<u64> {
    let number: u64 = number_str.parse()?;
    multiply_with_overflow_check(number, multiplier)
}

/// Multiply two values with overflow checking.
fn multiply_with_overflow_check(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| anyhow::anyhow!("Size value overflow: {a} * {b}"))
}

/// Add two values with overflow checking.
fn add_with_overflow_check(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| anyhow::anyhow!("Final overflow: {a} + {b}"))
}

/// Filter projects based on size and modification time criteria.
///
/// This function applies parallel filtering to remove projects that don't meet
/// the specified criteria:
/// - Projects smaller than the minimum size threshold
/// - Projects modified more recently than the specified number of days
///
/// # Arguments
///
/// * `projects` - Vector of projects to filter
/// * `filter_opts` - Filtering options containing size and time criteria
///
/// # Returns
///
/// - `Ok(Vec<Project>)` - Filtered list of projects that meet all criteria
/// - `Err(anyhow::Error)` - If size parsing fails, or file system errors occur
///
/// # Examples
///
/// ```
/// # use crate::{Project, FilterOptions};
/// # fn filter_projects(projects: Vec<Project>, opts: &FilterOptions) -> anyhow::Result<Vec<Project>> { Ok(vec![]) }
/// let filter_opts = FilterOptions {
///     keep_size: "100MB".to_string(),
///     keep_days: 30,
/// };
/// let filtered = filter_projects(projects, &filter_opts)?;
/// ```
fn filter_projects(projects: Vec<Project>, filter_opts: &FilterOptions) -> Result<Vec<Project>> {
    let keep_size_bytes = parse_size(&filter_opts.keep_size)?;
    let keep_days = filter_opts.keep_days;

    Ok(projects
        .into_par_iter()
        .filter(|project| meets_size_criteria(project, keep_size_bytes))
        .filter(|project| meets_time_criteria(project, keep_days))
        .collect())
}

/// Check if a project meets the size criteria.
fn meets_size_criteria(project: &Project, min_size: u64) -> bool {
    project.build_arts.size >= min_size
}

/// Check if a project meets the time criteria.
fn meets_time_criteria(project: &Project, keep_days: u32) -> bool {
    if keep_days == 0 {
        return true;
    }

    is_project_old_enough(project, keep_days)
}

/// Check if a project is old enough based on its modification time.
fn is_project_old_enough(project: &Project, keep_days: u32) -> bool {
    let Result::Ok(metadata) = fs::metadata(&project.build_arts.path) else {
        return true; // If we can't read metadata, don't filter it out
    };

    let Result::Ok(modified) = metadata.modified() else {
        return true; // If we can't read modification time, don't filter it out
    };

    let modified_time: DateTime<Local> = modified.into();
    let cutoff_time = Local::now() - chrono::Duration::days(i64::from(keep_days));

    modified_time <= cutoff_time
}
