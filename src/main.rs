mod cli;
mod project;
mod scanner;

use anyhow::{Ok, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use cli::{Cli, FilterOptions};
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use project::{Project, Projects};
use rayon::prelude::*;
use scanner::Scanner;
use std::{fs, process::exit};

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
    let execution_options = args.execution_options();
    let scan_options = args.scan_options();
    let filter_options = args.filter_options();

    println!("Cleaning directory: {}", dir.display());
    println!("Project filter: {project_filter:?}");
    println!("Execution options: {execution_options:?}");

    if scan_options.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(scan_options.threads)
            .build_global()
            .unwrap();
    }

    let scanner = Scanner::new(scan_options.verbose, scan_options.skip, project_filter);

    println!("Scanner: {scanner:?}");

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

    Ok(())
}

fn parse_size(size_str: &str) -> Result<u64> {
    if size_str == "0" {
        return Ok(0);
    }

    let size_str = size_str.to_uppercase();

    // Parse the numeric part and unit separately
    let (number_str, multiplier) = if size_str.ends_with("KB") {
        (size_str.trim_end_matches("KB"), 1_000u64)
    } else if size_str.ends_with("MB") {
        (size_str.trim_end_matches("MB"), 1_000_000u64)
    } else if size_str.ends_with("GB") {
        (size_str.trim_end_matches("GB"), 1_000_000_000u64)
    } else if size_str.ends_with("KIB") {
        (size_str.trim_end_matches("KIB"), 1_024u64)
    } else if size_str.ends_with("MIB") {
        (size_str.trim_end_matches("MIB"), 1_048_576u64)
    } else if size_str.ends_with("GIB") {
        (size_str.trim_end_matches("GIB"), 1_073_741_824u64)
    } else {
        (size_str.as_str(), 1u64)
    };

    // Handle decimal numbers by converting to fixed-point arithmetic
    if number_str.contains('.') {
        // Split into integer and fractional parts
        let parts: Vec<&str> = number_str.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid decimal format: {number_str}"));
        }

        let integer_part: u64 = parts[0].parse().unwrap_or(0);
        let fractional_str = parts[1];

        // Convert fractional part to integer (e.g., "5" -> 500000000 for 0.5)
        let fractional_digits = fractional_str.len();
        if fractional_digits > 9 {
            return Err(anyhow::anyhow!("Too many decimal places: {fractional_str}"));
        }

        let fractional_part: u64 = fractional_str.parse()?;
        let fractional_multiplier = if fractional_digits <= 9 {
            10u64.pow(9 - u32::try_from(fractional_digits)?)
        } else {
            return Err(anyhow::anyhow!(
                "Too many fractional digits: {fractional_digits}"
            ));
        };
        let fractional_normalized = fractional_part * fractional_multiplier;

        // Calculate: (integer_part * multiplier) + (fractional_part * multiplier / 10^9)
        let integer_result = integer_part
            .checked_mul(multiplier)
            .ok_or_else(|| anyhow::anyhow!("Integer overflow: {integer_part} * {multiplier}"))?;

        let fractional_result = fractional_normalized
            .checked_mul(multiplier)
            .ok_or_else(|| {
                anyhow::anyhow!("Fractional overflow: {fractional_normalized} * {multiplier}")
            })?
            / 1_000_000_000u64;

        integer_result
            .checked_add(fractional_result)
            .ok_or_else(|| {
                anyhow::anyhow!("Final overflow: {integer_result} + {fractional_result}")
            })
    } else {
        // For integer values, use integer arithmetic
        let number: u64 = number_str.parse()?;
        number
            .checked_mul(multiplier)
            .ok_or_else(|| anyhow::anyhow!("Size value overflow: {number} * {multiplier}"))
    }
}

fn filter_projects(projects: Vec<Project>, filter_opts: &FilterOptions) -> Result<Vec<Project>> {
    let keep_size_bytes = parse_size(&filter_opts.keep_size)?;
    let keep_days = filter_opts.keep_days;

    Ok(projects
        .into_par_iter()
        .filter(|project| {
            // Size filter
            if project.build_arts.size < keep_size_bytes {
                return false;
            }

            // Days filter
            if keep_days > 0 {
                if let core::result::Result::Ok(metadata) = fs::metadata(&project.build_arts.path) {
                    if let core::result::Result::Ok(modified) = metadata.modified() {
                        let modified_time: DateTime<Local> = modified.into();
                        let days_ago = Local::now() - chrono::Duration::days(i64::from(keep_days));

                        if modified_time > days_ago {
                            return false;
                        }
                    }
                }
            }

            true
        })
        .collect())
}
