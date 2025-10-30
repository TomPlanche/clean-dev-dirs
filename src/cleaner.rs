//! Build directory cleanup functionality.
//!
//! This module provides the core cleanup logic for removing build directories
//! from detected development projects. It handles parallel processing, progress
//! reporting, error handling, and provides detailed statistics about the
//! cleanup operation.

use anyhow::Result;
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::config::ExecutionOptions;
use crate::project::{Project, ProjectType, Projects};

/// Handles the cleanup of build directories from development projects.
///
/// The `Cleaner` struct provides methods for removing build directories
/// (such as `target/` for Rust projects and `node_modules/` for Node.js projects)
/// with parallel processing, progress reporting, and comprehensive error handling.
pub struct Cleaner;

impl Cleaner {
    /// Create a new cleaner instance.
    ///
    /// # Returns
    ///
    /// A new `Cleaner` instance ready to perform cleanup operations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::Cleaner;
    /// let cleaner = Cleaner::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Clean build directories from a collection of projects.
    ///
    /// This method performs the main cleanup operation by:
    /// 1. Setting up a progress bar for user feedback
    /// 2. Processing projects in parallel for efficiency
    /// 3. Collecting and reporting any errors that occur
    /// 4. Providing detailed statistics about the cleanup results
    ///
    /// # Arguments
    ///
    /// * `projects` - A collection of projects to clean
    /// * `execution_options` - Configuration options for the cleanup operation
    ///
    /// # Panics
    ///
    /// This method may panic if the progress bar template string is invalid,
    /// though this should not occur under normal circumstances as the template
    /// is hardcoded and valid.
    ///
    /// # Output
    ///
    /// This method prints progress information and final statistics to stdout,
    /// including
    /// - Real-time progress during cleanup
    /// - Number of successfully cleaned projects
    /// - Number of failed projects (if any)
    /// - Total disk space freed
    /// - Difference between estimated and actual space freed
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::{Cleaner, Projects, ExecutionOptions};
    /// let projects = Projects::from(vec![/* project instances */]);
    /// let options = ExecutionOptions {
    ///     dry_run: false,
    ///     interactive: false,
    ///     keep_executables: true,
    /// };
    /// Cleaner::clean_projects(projects, &options);
    /// ```
    ///
    /// # Performance
    ///
    /// This method uses parallel processing to clean multiple projects
    /// simultaneously, which can significantly reduce cleanup time for
    /// large numbers of projects.
    ///
    /// # Error Handling
    ///
    /// Individual project cleanup failures do not stop the overall process.
    /// All errors are collected and reported at the end, allowing the
    /// cleanup to proceed for projects that can be successfully processed.
    pub fn clean_projects(projects: Projects, execution_options: &ExecutionOptions) {
        let total_projects = projects.len();
        let total_size: u64 = projects.get_total_size();

        println!("\n{}", "🧹 Starting cleanup...".cyan());

        // Create a progress bar
        let progress = ProgressBar::new(total_projects as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let cleaned_size = Arc::new(Mutex::new(0u64));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Clean projects in parallel
        projects.into_par_iter().for_each(|project| {
            let result = clean_single_project(&project, execution_options.keep_executables);

            match result {
                Ok(freed_size) => {
                    let mut total_cleaned = cleaned_size.lock().unwrap();
                    *total_cleaned += freed_size;

                    progress.set_message(format!(
                        "Cleaned {} ({})",
                        project
                            .root_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown"),
                        format_size(freed_size, DECIMAL)
                    ));
                }
                Err(e) => {
                    let mut errors = errors.lock().unwrap();
                    errors.push(format!(
                        "Failed to clean {}: {e}",
                        project.build_arts.path.display()
                    ));
                }
            }

            progress.inc(1);
        });

        progress.finish_with_message("✅ Cleanup complete");

        // Report results
        let final_cleaned_size = *cleaned_size.lock().unwrap();
        let errors = errors.lock().unwrap();

        if !errors.is_empty() {
            println!("\n{}", "⚠️  Some errors occurred during cleanup:".yellow());
            for error in errors.iter() {
                eprintln!("  {}", error.red());
            }
        }

        let success_count = total_projects - errors.len();
        println!("\n{}", "📊 Cleanup Summary:".bold());
        println!(
            "  ✅ Successfully cleaned: {} projects",
            success_count.to_string().green()
        );

        if !errors.is_empty() {
            println!(
                "  ❌ Failed to clean: {} projects",
                errors.len().to_string().red()
            );
        }

        println!(
            "  💾 Total space freed: {}",
            format_size(final_cleaned_size, DECIMAL)
                .bright_green()
                .bold()
        );

        if final_cleaned_size != total_size {
            let difference = total_size.abs_diff(final_cleaned_size);
            println!(
                "  📋 Difference from estimate: {}",
                format_size(difference, DECIMAL).yellow()
            );
        }
    }
}

/// Clean a build directory while preserving executable binaries.
///
/// This function implements selective deletion for Rust and Go projects,
/// preserving final executable binaries while removing intermediate build artifacts.
///
/// # Arguments
///
/// * `project` - The project to clean
///
/// # Returns
///
/// - `Ok(())` - If the cleanup succeeded
/// - `Err(anyhow::Error)` - If the cleanup operation failed
///
/// # Behavior by Project Type
///
/// **Rust Projects:**
/// - Preserves executables in `target/debug/` and `target/release/` directories
/// - Removes all other files and directories within `target/`
///
/// **Go Projects:**
/// - Preserves executables in `bin/` directory if it exists within the build dir
/// - Removes all other files and directories
///
/// **Other Project Types:**
/// - Falls back to complete removal (no executables to preserve)
fn clean_with_executable_preservation(project: &Project) -> Result<()> {
    match project.kind {
        ProjectType::Rust => clean_rust_with_executables(project),
        ProjectType::Go => clean_go_with_executables(project),
        _ => {
            // For Node.js and Python, there are no compiled executables to preserve
            fs::remove_dir_all(&project.build_arts.path)?;
            Ok(())
        }
    }
}

/// Clean a Rust project's target directory while preserving executables.
///
/// This function identifies executable files in the `target/debug/` and `target/release/`
/// directories and preserves them while removing all other build artifacts.
///
/// # Arguments
///
/// * `project` - The Rust project to clean
///
/// # Returns
///
/// - `Ok(())` - If the cleanup succeeded
/// - `Err(anyhow::Error)` - If the cleanup operation failed
///
/// # Implementation
///
/// 1. Scans `target/debug/` and `target/release/` for executable files
/// 2. Backs up found executables to a temporary location
/// 3. Removes the entire `target/` directory
/// 4. Recreates `target/debug/` and `target/release/` directories
/// 5. Restores the executables to their original locations
fn clean_rust_with_executables(project: &Project) -> Result<()> {
    let target_dir = &project.build_arts.path;
    let debug_dir = target_dir.join("debug");
    let release_dir = target_dir.join("release");

    // Find executables in debug and release directories
    let mut executables = Vec::new();

    for dir in [&debug_dir, &release_dir] {
        if dir.exists()
            && let Ok(entries) = fs::read_dir(dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && is_executable(&path) {
                    executables.push(path);
                }
            }
        }
    }

    // If no executables found, just remove everything
    if executables.is_empty() {
        fs::remove_dir_all(target_dir)?;
        return Ok(());
    }

    // Create temporary directory for backup
    let temp_dir =
        std::env::temp_dir().join(format!("clean-dev-dirs-backup-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    // Backup executables
    let mut backed_up = Vec::new();
    for exe in &executables {
        if let Some(file_name) = exe.file_name() {
            let backup_path = temp_dir.join(file_name);
            if fs::copy(exe, &backup_path).is_ok() {
                backed_up.push((backup_path, exe.clone()));
            }
        }
    }

    // Remove the target directory
    fs::remove_dir_all(target_dir)?;

    // Recreate debug and release directories and restore executables
    for (backup_path, original_path) in backed_up {
        if let Some(parent) = original_path.parent() {
            fs::create_dir_all(parent)?;
            let _ = fs::copy(&backup_path, &original_path);
        }
    }

    // Clean up temporary directory
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Clean a Go project's vendor directory while preserving executables.
///
/// This function preserves executable files in the `bin/` directory within
/// the vendor directory, if it exists.
///
/// # Arguments
///
/// * `project` - The Go project to clean
///
/// # Returns
///
/// - `Ok(())` - If the cleanup succeeded
/// - `Err(anyhow::Error)` - If the cleanup operation failed
fn clean_go_with_executables(project: &Project) -> Result<()> {
    let vendor_dir = &project.build_arts.path;
    let bin_dir = vendor_dir.join("bin");

    // Check if there's a bin directory with executables
    let mut executables = Vec::new();
    if bin_dir.exists()
        && let Ok(entries) = fs::read_dir(&bin_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_executable(&path) {
                executables.push(path);
            }
        }
    }

    // If no executables found, just remove everything
    if executables.is_empty() {
        fs::remove_dir_all(vendor_dir)?;
        return Ok(());
    }

    // Create temporary directory for backup
    let temp_dir =
        std::env::temp_dir().join(format!("clean-dev-dirs-backup-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    // Backup executables
    let mut backed_up = Vec::new();
    for exe in &executables {
        if let Some(file_name) = exe.file_name() {
            let backup_path = temp_dir.join(file_name);
            if fs::copy(exe, &backup_path).is_ok() {
                backed_up.push((backup_path, exe.clone()));
            }
        }
    }

    // Remove the vendor directory
    fs::remove_dir_all(vendor_dir)?;

    // Recreate bin directory and restore executables
    if !backed_up.is_empty() {
        fs::create_dir_all(&bin_dir)?;
        for (backup_path, original_path) in backed_up {
            let _ = fs::copy(&backup_path, &original_path);
        }
    }

    // Clean up temporary directory
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Check if a file is executable.
///
/// On Unix systems, checks the executable permission bit.
/// On Windows, checks for common executable extensions (.exe, .dll, .com).
///
/// # Arguments
///
/// * `path` - Path to the file to check
///
/// # Returns
///
/// `true` if the file is likely an executable, `false` otherwise
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            // Check if any execute bit is set
            return permissions.mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return matches!(ext_str.as_str(), "exe" | "dll" | "com");
        }
        false
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback for other platforms
        false
    }
}

/// Clean the build directory for a single project.
///
/// This function handles the cleanup of an individual project's build directory.
/// It calculates the actual size before deletion and then removes the entire
/// directory tree, optionally preserving executable binaries.
///
/// # Arguments
///
/// * `project` - The project whose build directory should be cleaned
/// * `keep_executables` - Whether to preserve compiled executables
///
/// # Returns
///
/// - `Ok(u64)` - The number of bytes freed by the cleanup
/// - `Err(anyhow::Error)` - If the cleanup operation failed
///
/// # Behavior
///
/// 1. Checks if the build directory exists (returns 0 if not)
/// 2. Calculates the actual size of the directory before deletion
/// 3. If `keep_executables` is true and the project is Rust or Go:
///    - Preserves executable binaries in appropriate locations
///    - Removes only intermediate build artifacts
/// 4. Otherwise, removes the entire directory tree
/// 5. Returns the amount of space freed
///
/// # Error Conditions
///
/// This function can fail if:
/// - The build directory cannot be removed due to permission issues
/// - Files within the directory are locked or in use by other processes
/// - The file system encounters I/O errors during deletion
///
/// # Examples
///
/// ```
/// # use crate::{Project, clean_single_project};
/// # use anyhow::Result;
/// let result = clean_single_project(&project, true);
/// match result {
///     Ok(freed_bytes) => println!("Freed {} bytes", freed_bytes),
///     Err(e) => eprintln!("Cleanup failed: {}", e),
/// }
/// ```
fn clean_single_project(project: &Project, keep_executables: bool) -> Result<u64> {
    let build_dir = &project.build_arts.path;

    if !build_dir.exists() {
        return Ok(0);
    }

    // Get the actual size before deletion (might be different from the cached size)
    let actual_size = calculate_directory_size(build_dir);

    // Remove the build directory with optional executable preservation
    if keep_executables {
        clean_with_executable_preservation(project)?;
    } else {
        fs::remove_dir_all(build_dir)?;
    }

    Ok(actual_size)
}

/// Calculate the total size of a directory and all its contents.
///
/// This function recursively traverses a directory tree and sums up the sizes
/// of all files within it. It handles errors gracefully by skipping files
/// that cannot be accessed.
///
/// # Arguments
///
/// * `path` - The directory path to measure
///
/// # Returns
///
/// The total size of all files in the directory tree, in bytes.
///
/// # Error Handling
///
/// This function is designed to be robust and will continue processing even
/// if individual files cannot be accessed. It silently skips:
/// - Files that cannot be read due to permission issues
/// - Broken symbolic links
/// - Files that are deleted while the scan is in progress
///
/// # Performance
///
/// This function can be I/O intensive for large directories with many files.
/// It processes files sequentially within each directory but may be called
/// in parallel for different directories by the cleanup process.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use crate::calculate_directory_size;
/// let size = calculate_directory_size(Path::new("/path/to/directory"));
/// println!("Directory size: {} bytes", size);
/// ```
fn calculate_directory_size(path: &std::path::Path) -> u64 {
    let mut total_size = 0u64;

    for entry in walkdir::WalkDir::new(path) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file()
                && let Ok(metadata) = entry.metadata()
            {
                total_size += metadata.len();
            }
        } else {
            // Skip errors for individual files
        }
    }

    total_size
}

impl Default for Cleaner {
    /// Create a default cleaner instance.
    ///
    /// This implementation allows `Cleaner::default()` to be used as an
    /// alternative to `Cleaner::new()` for creating cleaner instances.
    ///
    /// # Returns
    ///
    /// A new `Cleaner` instance with default settings.
    fn default() -> Self {
        Self::new()
    }
}
