//! Directory scanning and project detection functionality.
//!
//! This module provides the core scanning logic that traverses directory trees
//! to find development projects and their build artifacts. It supports parallel
//! processing for improved performance and handles various error conditions
//! gracefully.

use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::{Value, from_str};
use walkdir::{DirEntry, WalkDir};

use crate::{
    cli::{ProjectFilter, ScanOptions},
    project::{BuildArtifacts, Project, ProjectType},
};

/// Directory scanner for detecting development projects.
///
/// The `Scanner` struct encapsulates the logic for traversing directory trees
/// and identifying development projects (Rust and Node.js) along with their
/// build artifacts. It supports configurable filtering and parallel processing
/// for efficient scanning of large directory structures.
pub(crate) struct Scanner {
    /// Configuration options for scanning behavior
    scan_options: ScanOptions,
    
    /// Filter to restrict scanning to specific project types
    project_filter: ProjectFilter,
}

impl Scanner {
    /// Create a new scanner with the specified options.
    ///
    /// # Arguments
    ///
    /// * `scan_options` - Configuration for scanning behavior (threads, verbosity, etc.)
    /// * `project_filter` - Filter to restrict scanning to specific project types
    ///
    /// # Returns
    ///
    /// A new `Scanner` instance configured with the provided options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::{Scanner, ScanOptions, ProjectFilter};
    /// let scan_options = ScanOptions {
    ///     verbose: true,
    ///     threads: 4,
    ///     skip: vec![],
    /// };
    /// 
    /// let scanner = Scanner::new(scan_options, ProjectFilter::All);
    /// ```
    pub(crate) fn new(scan_options: ScanOptions, project_filter: ProjectFilter) -> Self {
        Self {
            scan_options,
            project_filter,
        }
    }

    /// Scan a directory tree for development projects.
    ///
    /// This method performs a recursive scan of the specified directory to find
    /// development projects. It operates in two phases:
    /// 1. Directory traversal to identify potential projects
    /// 2. Parallel size calculation for build directories
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory to start scanning from
    ///
    /// # Returns
    ///
    /// A vector of `Project` instances representing all detected projects with
    /// non-zero build directory sizes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use crate::Scanner;
    /// let projects = scanner.scan_directory(Path::new("/path/to/projects"));
    /// println!("Found {} projects", projects.len());
    /// ```
    ///
    /// # Performance
    ///
    /// This method uses parallel processing for both directory traversal and
    /// size calculation to maximize performance on systems with multiple cores
    /// and fast storage.
    pub(crate) fn scan_directory(&self, root: &Path) -> Vec<Project> {
        let errors = Arc::new(Mutex::new(Vec::<String>::new()));

        // Create a progress bar
        let progress = ProgressBar::new_spinner();
        progress.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        progress.set_message("Scanning directories...");

        // Find all potential project directories
        let potential_projects: Vec<_> = WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| self.should_scan_entry(entry))
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter_map(|entry| self.detect_project(&entry, &errors))
            .collect();

        progress.finish_with_message("✅ Directory scan complete");

        // Process projects in parallel to calculate sizes
        let projects_with_sizes: Vec<_> = potential_projects
            .into_par_iter()
            .filter_map(|mut project| {
                let size = self.calculate_build_dir_size(&project.build_arts.path);
                project.build_arts.size = size;

                if size > 0 { Some(project) } else { None }
            })
            .collect();

        // Print errors if verbose
        if self.scan_options.verbose {
            let errors = errors.lock().unwrap();
            for error in errors.iter() {
                eprintln!("{}", error.red());
            }
        }

        projects_with_sizes
    }

    /// Calculate the total size of a build directory.
    ///
    /// This method recursively traverses the specified directory and sums up
    /// the sizes of all files contained within it. It handles errors gracefully
    /// and optionally reports them in verbose mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the build directory to measure
    ///
    /// # Returns
    ///
    /// The total size of all files in the directory, in bytes. Returns 0 if
    /// the directory doesn't exist or cannot be accessed.
    ///
    /// # Performance
    ///
    /// This method can be CPU and I/O intensive for large directories with
    /// many files. It's designed to be called in parallel for multiple
    /// directories to maximize throughput.
    fn calculate_build_dir_size(&self, path: &Path) -> u64 {
        if !path.exists() {
            return 0;
        }

        let mut total_size = 0u64;

        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Ok(metadata) = entry.metadata() {
                            total_size += metadata.len();
                        }
                    }
                }
                Err(e) => {
                    if self.scan_options.verbose {
                        eprintln!("Warning: {e}");
                    }
                }
            }
        }

        total_size
    }

    /// Detect if a directory entry represents a development project.
    ///
    /// This method examines a directory entry and determines if it contains
    /// a development project based on the presence of characteristic files
    /// and directories. It respects the project filter settings.
    ///
    /// # Arguments
    ///
    /// * `entry` - The directory entry to examine
    /// * `errors` - Shared error collection for reporting issues
    ///
    /// # Returns
    ///
    /// - `Some(Project)` if a valid project is detected
    /// - `None` if no project is found or the entry doesn't match filters
    ///
    /// # Project Detection Logic
    ///
    /// - **Rust projects**: Presence of both `Cargo.toml` and `target/` directory
    /// - **Node.js projects**: Presence of both `package.json` and `node_modules/` directory
    fn detect_project(
        &self,
        entry: &DirEntry,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<Project> {
        let path = entry.path();

        if !entry.file_type().is_dir() {
            return None;
        }

        // Check for a Rust project
        if matches!(
            self.project_filter,
            ProjectFilter::All | ProjectFilter::RustOnly
        ) {
            if let Some(project) = self.detect_rust_project(path, errors) {
                return Some(project);
            }
        }

        // Check for a Node.js project
        if matches!(
            self.project_filter,
            ProjectFilter::All | ProjectFilter::NodeOnly
        ) {
            if let Some(project) = self.detect_node_project(path, errors) {
                return Some(project);
            }
        }

        None
    }

    /// Detect a Rust project in the specified directory.
    ///
    /// This method checks for the presence of both `Cargo.toml` and `target/`
    /// directory to identify a Rust project. If found, it attempts to extract
    /// the project name from the `Cargo.toml` file.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to check for a Rust project
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(Project)` if a valid Rust project is detected
    /// - `None` if the directory doesn't contain a Rust project
    ///
    /// # Detection Criteria
    ///
    /// 1. `Cargo.toml` file exists in directory
    /// 2. `target/` subdirectory exists in directory
    /// 3. The project name is extracted from `Cargo.toml` if possible
    fn detect_rust_project(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<Project> {
        let cargo_toml = path.join("Cargo.toml");
        let target_dir = path.join("target");

        if cargo_toml.exists() && target_dir.exists() {
            let name = self.extract_rust_project_name(&cargo_toml, errors);

            let build_arts = BuildArtifacts {
                path: path.join("target"),
                size: 0, // Will be calculated later
            };

            return Some(Project::new(
                ProjectType::Rust,
                path.to_path_buf(),
                build_arts,
                name,
            ));
        }

        None
    }

    /// Detect a Node.js project in the specified directory.
    ///
    /// This method checks for the presence of both `package.json` and `node_modules/`
    /// directory to identify a Node.js project. If found, it attempts to extract
    /// the project name from the `package.json` file.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to check for Node.js project
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(Project)` if a valid Node.js project is detected
    /// - `None` if the directory doesn't contain a Node.js project
    ///
    /// # Detection Criteria
    ///
    /// 1. `package.json` file exists in directory
    /// 2. `node_modules/` subdirectory exists in directory
    /// 3. The project name is extracted from `package.json` if possible
    fn detect_node_project(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<Project> {
        let package_json = path.join("package.json");
        let node_modules = path.join("node_modules");

        if package_json.exists() && node_modules.exists() {
            let name = self.extract_node_project_name(&package_json, errors);

            let build_arts = BuildArtifacts {
                path: path.join("node_modules"),
                size: 0, // Will be calculated later
            };

            return Some(Project::new(
                ProjectType::Node,
                path.to_path_buf(),
                build_arts,
                name,
            ));
        }

        None
    }

    /// Extract the project name from a Cargo.toml file.
    ///
    /// This method performs simple TOML parsing to extract the project name
    /// from a Rust project's `Cargo.toml` file. It uses a line-by-line approach
    /// rather than a full TOML parser for simplicity and performance.
    ///
    /// # Arguments
    ///
    /// * `cargo_toml` - Path to the Cargo.toml file
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(String)` containing the project name if successfully extracted
    /// - `None` if the name cannot be found or parsed
    ///
    /// # Parsing Strategy
    ///
    /// The method looks for lines matching the pattern `name = "project_name"`
    /// and extracts the quoted string value. This trivial approach handles
    /// most common cases without requiring a full TOML parser.
    fn extract_rust_project_name(
        &self,
        cargo_toml: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        match fs::read_to_string(cargo_toml) {
            Ok(content) => {
                // Simple TOML parsing - look for name = "..."
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("name") && line.contains('=') {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line.rfind('"') {
                                if start != end {
                                    return Some(line[start + 1..end].to_string());
                                }
                            }
                        }
                    }
                }
                None
            }
            Err(e) => {
                if self.scan_options.verbose {
                    let mut errors = errors.lock().unwrap();
                    errors.push(format!("Error reading {}: {e}", cargo_toml.display()));
                }
                None
            }
        }
    }

    /// Extract the project name from a package.json file.
    ///
    /// This method parses a Node.js project's `package.json` file to extract
    /// the project name. It uses full JSON parsing to handle the file format
    /// correctly and safely.
    ///
    /// # Arguments
    ///
    /// * `package_json` - Path to the package.json file
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(String)` containing the project name if successfully extracted
    /// - `None` if the name cannot be found, parsed, or the file is invalid
    ///
    /// # Error Handling
    ///
    /// This method handles both file I/O errors and JSON parsing errors gracefully.
    /// Errors are optionally reported to the shared error collection in verbose mode.
    fn extract_node_project_name(
        &self,
        package_json: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        match fs::read_to_string(package_json) {
            Ok(content) => match from_str::<Value>(&content) {
                Ok(json) => json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string),
                Err(e) => {
                    if self.scan_options.verbose {
                        let mut errors = errors.lock().unwrap();
                        errors.push(format!("Error parsing {}: {e}", package_json.display()));
                    }
                    None
                }
            },
            Err(e) => {
                if self.scan_options.verbose {
                    let mut errors = errors.lock().unwrap();
                    errors.push(format!("Error reading {}: {e}", package_json.display()));
                }
                None
            }
        }
    }

    /// Determine if a directory entry should be scanned for projects.
    ///
    /// This method implements the filtering logic to decide whether a directory
    /// should be traversed during the scanning process. It applies various
    /// exclusion rules to improve performance and avoid scanning irrelevant
    /// directories.
    ///
    /// # Arguments
    ///
    /// * `entry` - The directory entry to evaluate
    ///
    /// # Returns
    ///
    /// - `true` if the directory should be scanned
    /// - `false` if the directory should be skipped
    ///
    /// # Exclusion Rules
    ///
    /// The following directories are excluded from scanning:
    /// - Directories in the user-specified skip list
    /// - Any directory inside a `node_modules/` directory (to avoid deep nesting)
    /// - Hidden directories (starting with `.`) except `.cargo`
    /// - Common build/temporary directories: `target`, `build`, `dist`, `out`, etc.
    /// - Version control directories: `.git`, `.svn`, `.hg`
    /// - Python cache and virtual environment directories
    /// - Temporary directories: `temp`, `tmp`
    ///
    /// # Performance Impact
    ///
    /// This filtering significantly reduces the number of directories that need
    /// to be processed, especially in large codebases with many build artifacts
    /// or deeply nested `node_modules` directories.
    fn should_scan_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        // Skip directories in the skip list
        if self
            .scan_options
            .skip
            .iter()
            .any(|skip| path.starts_with(skip))
        {
            return false;
        }

        // Skip any directory inside a node_modules directory
        if path.ancestors().any(|ancestor| {
            ancestor
                .file_name()
                .and_then(|n| n.to_str()) == Some("node_modules")
        }) {
            return false;
        }

        // Skip hidden directories (except .cargo for Rust)
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') && name != ".cargo" {
                return false;
            }
        }

        // Skip common non-project directories
        let excluded_dirs = [
            "target",
            "build",
            "dist",
            "out",
            ".git",
            ".svn",
            ".hg",
            "__pycache__",
            "venv",
            ".venv",
            "env",
            ".env",
            "temp",
            "tmp",
        ];

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if excluded_dirs.contains(&name) {
                return false;
            }
        }

        true
    }
}
