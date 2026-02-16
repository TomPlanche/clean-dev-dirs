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
    config::{ProjectFilter, ScanOptions},
    project::{BuildArtifacts, Project, ProjectType},
};

/// Directory scanner for detecting development projects.
///
/// The `Scanner` struct encapsulates the logic for traversing directory trees
/// and identifying development projects (Rust and Node.js) along with their
/// build artifacts. It supports configurable filtering and parallel processing
/// for efficient scanning of large directory structures.
pub struct Scanner {
    /// Configuration options for scanning behavior
    scan_options: ScanOptions,

    /// Filter to restrict scanning to specific project types
    project_filter: ProjectFilter,

    /// When `true`, suppresses progress spinner output (used by `--json` mode).
    quiet: bool,
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
    #[must_use]
    pub const fn new(scan_options: ScanOptions, project_filter: ProjectFilter) -> Self {
        Self {
            scan_options,
            project_filter,
            quiet: false,
        }
    }

    /// Enable or disable quiet mode (suppresses progress spinner).
    ///
    /// When quiet mode is active the scanning spinner is hidden, which is
    /// required for `--json` output so that only the final JSON is printed.
    #[must_use]
    pub const fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
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
    /// # Panics
    ///
    /// This method may panic if the progress bar template string is invalid,
    /// though this should not occur under normal circumstances as the template
    /// is hardcoded and valid.
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
    pub fn scan_directory(&self, root: &Path) -> Vec<Project> {
        let errors = Arc::new(Mutex::new(Vec::<String>::new()));

        let progress = if self.quiet {
            ProgressBar::hidden()
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb.set_message("Scanning directories...");
            pb
        };

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
                    if entry.file_type().is_file()
                        && let Ok(metadata) = entry.metadata()
                    {
                        total_size += metadata.len();
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
    /// - **Python projects**: Presence of configuration files and cache directories
    /// - **Go projects**: Presence of both `go.mod` and `vendor/` directory
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
            ProjectFilter::All | ProjectFilter::Rust
        ) && let Some(project) = self.detect_rust_project(path, errors)
        {
            return Some(project);
        }

        // Check for a Node.js project
        if matches!(
            self.project_filter,
            ProjectFilter::All | ProjectFilter::Node
        ) && let Some(project) = self.detect_node_project(path, errors)
        {
            return Some(project);
        }

        // Check for a Python project
        if matches!(
            self.project_filter,
            ProjectFilter::All | ProjectFilter::Python
        ) && let Some(project) = self.detect_python_project(path, errors)
        {
            return Some(project);
        }

        // Check for a Go project
        if matches!(self.project_filter, ProjectFilter::All | ProjectFilter::Go)
            && let Some(project) = self.detect_go_project(path, errors)
        {
            return Some(project);
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
        let content = self.read_file_content(cargo_toml, errors)?;
        Self::parse_toml_name_field(&content)
    }

    /// Extract a quoted string value from a line.
    fn extract_quoted_value(line: &str) -> Option<String> {
        let start = line.find('"')?;
        let end = line.rfind('"')?;

        if start == end {
            return None;
        }

        Some(line[start + 1..end].to_string())
    }

    /// Extract the name from a single TOML line if it contains a name field.
    fn extract_name_from_line(line: &str) -> Option<String> {
        if !Self::is_name_line(line) {
            return None;
        }

        Self::extract_quoted_value(line)
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
                        errors
                            .lock()
                            .unwrap()
                            .push(format!("Error parsing {}: {e}", package_json.display()));
                    }
                    None
                }
            },
            Err(e) => {
                if self.scan_options.verbose {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("Error reading {}: {e}", package_json.display()));
                }
                None
            }
        }
    }

    /// Check if a line contains a name field assignment.
    fn is_name_line(line: &str) -> bool {
        line.starts_with("name") && line.contains('=')
    }

    /// Log a file reading error if verbose mode is enabled.
    fn log_file_error(
        &self,
        file_path: &Path,
        error: &std::io::Error,
        errors: &Arc<Mutex<Vec<String>>>,
    ) {
        if self.scan_options.verbose {
            errors
                .lock()
                .unwrap()
                .push(format!("Error reading {}: {error}", file_path.display()));
        }
    }

    /// Parse the name field from TOML content.
    fn parse_toml_name_field(content: &str) -> Option<String> {
        for line in content.lines() {
            if let Some(name) = Self::extract_name_from_line(line.trim()) {
                return Some(name);
            }
        }
        None
    }

    /// Read the content of a file and handle errors appropriately.
    fn read_file_content(
        &self,
        file_path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        match fs::read_to_string(file_path) {
            Ok(content) => Some(content),
            Err(e) => {
                self.log_file_error(file_path, &e, errors);
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
    /// - Go vendor directory
    /// - Python pytest cache
    /// - Python tox environments
    /// - Python setuptools
    /// - Python coverage files
    /// - Node.js modules (already handled above but added for completeness)
    fn should_scan_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        // Early return if path is in skip list
        if self.is_path_in_skip_list(path) {
            return false;
        }

        // Skip any directory inside a node_modules directory
        if path
            .ancestors()
            .any(|ancestor| ancestor.file_name().and_then(|n| n.to_str()) == Some("node_modules"))
        {
            return false;
        }

        // Skip hidden directories (except .cargo for Rust)
        if Self::is_hidden_directory_to_skip(path) {
            return false;
        }

        // Skip common non-project directories
        !Self::is_excluded_directory(path)
    }

    /// Check if a path is in the skip list
    fn is_path_in_skip_list(&self, path: &Path) -> bool {
        self.scan_options.skip.iter().any(|skip| {
            path.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|name| name == skip.to_string_lossy())
            })
        })
    }

    /// Check if directory is hidden and should be skipped
    fn is_hidden_directory_to_skip(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with('.') && name != ".cargo")
    }

    /// Check if directory is in the excluded list
    fn is_excluded_directory(path: &Path) -> bool {
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
            "vendor",
            ".pytest_cache",
            ".tox",
            ".eggs",
            ".coverage",
            "node_modules",
        ];

        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| excluded_dirs.contains(&name))
    }

    /// Detect a Python project in the specified directory.
    ///
    /// This method checks for Python configuration files and associated cache directories.
    /// It looks for multiple build artifacts that can be cleaned.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to check for a Python project
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(Project)` if a valid Python project is detected
    /// - `None` if the directory doesn't contain a Python project
    ///
    /// # Detection Criteria
    ///
    /// A Python project is identified by having:
    /// 1. At least one of: requirements.txt, setup.py, pyproject.toml, setup.cfg, Pipfile
    /// 2. At least one of the cache/build directories: `__pycache__`, `.pytest_cache`, venv, .venv, build, dist, .eggs
    fn detect_python_project(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<Project> {
        let config_files = [
            "requirements.txt",
            "setup.py",
            "pyproject.toml",
            "setup.cfg",
            "Pipfile",
            "pipenv.lock",
            "poetry.lock",
        ];

        let build_dirs = [
            "__pycache__",
            ".pytest_cache",
            "venv",
            ".venv",
            "build",
            "dist",
            ".eggs",
            ".tox",
            ".coverage",
        ];

        // Check if any config file exists
        let has_config = config_files.iter().any(|&file| path.join(file).exists());

        if !has_config {
            return None;
        }

        // Find the largest cache/build directory that exists
        let mut largest_build_dir = None;
        let mut largest_size = 0;

        for &dir_name in &build_dirs {
            let dir_path = path.join(dir_name);

            if dir_path.exists()
                && dir_path.is_dir()
                && let Ok(size) = Self::calculate_directory_size(&dir_path)
                && size > largest_size
            {
                largest_size = size;
                largest_build_dir = Some(dir_path);
            }
        }

        if let Some(build_path) = largest_build_dir {
            let name = self.extract_python_project_name(path, errors);

            let build_arts = BuildArtifacts {
                path: build_path,
                size: 0, // Will be calculated later
            };

            return Some(Project::new(
                ProjectType::Python,
                path.to_path_buf(),
                build_arts,
                name,
            ));
        }

        None
    }

    /// Detect a Go project in the specified directory.
    ///
    /// This method checks for the presence of both `go.mod` and `vendor/`
    /// directory to identify a Go project. If found, it attempts to extract
    /// the project name from the `go.mod` file.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to check for a Go project
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(Project)` if a valid Go project is detected
    /// - `None` if the directory doesn't contain a Go project
    ///
    /// # Detection Criteria
    ///
    /// 1. `go.mod` file exists in directory
    /// 2. `vendor/` subdirectory exists in directory
    /// 3. The project name is extracted from `go.mod` if possible
    fn detect_go_project(&self, path: &Path, errors: &Arc<Mutex<Vec<String>>>) -> Option<Project> {
        let go_mod = path.join("go.mod");
        let vendor_dir = path.join("vendor");

        if go_mod.exists() && vendor_dir.exists() {
            let name = self.extract_go_project_name(&go_mod, errors);

            let build_arts = BuildArtifacts {
                path: path.join("vendor"),
                size: 0, // Will be calculated later
            };

            return Some(Project::new(
                ProjectType::Go,
                path.to_path_buf(),
                build_arts,
                name,
            ));
        }

        None
    }

    /// Extract the project name from a Python project directory.
    ///
    /// This method attempts to extract the project name from various Python
    /// configuration files in order of preference.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Python project directory
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(String)` containing the project name if successfully extracted
    /// - `None` if the name cannot be found or parsed
    ///
    /// # Extraction Order
    ///
    /// 1. pyproject.toml (from [project] name or [tool.poetry] name)
    /// 2. setup.py (from name= parameter)
    /// 3. setup.cfg (from [metadata] name)
    /// 4. Use directory name as a fallback
    fn extract_python_project_name(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        // Try files in order of preference
        self.try_extract_from_pyproject_toml(path, errors)
            .or_else(|| self.try_extract_from_setup_py(path, errors))
            .or_else(|| self.try_extract_from_setup_cfg(path, errors))
            .or_else(|| Self::fallback_to_directory_name(path))
    }

    /// Try to extract project name from pyproject.toml
    fn try_extract_from_pyproject_toml(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        let pyproject_toml = path.join("pyproject.toml");
        if !pyproject_toml.exists() {
            return None;
        }

        let content = self.read_file_content(&pyproject_toml, errors)?;
        Self::extract_name_from_toml_like_content(&content)
    }

    /// Try to extract project name from setup.py
    fn try_extract_from_setup_py(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        let setup_py = path.join("setup.py");
        if !setup_py.exists() {
            return None;
        }

        let content = self.read_file_content(&setup_py, errors)?;
        Self::extract_name_from_python_content(&content)
    }

    /// Try to extract project name from setup.cfg
    fn try_extract_from_setup_cfg(
        &self,
        path: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        let setup_cfg = path.join("setup.cfg");
        if !setup_cfg.exists() {
            return None;
        }

        let content = self.read_file_content(&setup_cfg, errors)?;
        Self::extract_name_from_cfg_content(&content)
    }

    /// Extract name from TOML-like content (pyproject.toml)
    fn extract_name_from_toml_like_content(content: &str) -> Option<String> {
        content
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("name") && line.contains('='))
            .and_then(Self::extract_quoted_value)
    }

    /// Extract name from Python content (setup.py)
    fn extract_name_from_python_content(content: &str) -> Option<String> {
        content
            .lines()
            .map(str::trim)
            .find(|line| line.contains("name") && line.contains('='))
            .and_then(Self::extract_quoted_value)
    }

    /// Extract name from INI-style configuration content (setup.cfg)
    fn extract_name_from_cfg_content(content: &str) -> Option<String> {
        let mut in_metadata_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line == "[metadata]" {
                in_metadata_section = true;
            } else if line.starts_with('[') && line.ends_with(']') {
                in_metadata_section = false;
            } else if in_metadata_section && line.starts_with("name") && line.contains('=') {
                return line.split('=').nth(1).map(|name| name.trim().to_string());
            }
        }

        None
    }

    /// Fallback to directory name
    fn fallback_to_directory_name(path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(std::string::ToString::to_string)
    }

    /// Extract the project name from a `go.mod` file.
    ///
    /// This method parses a Go project's `go.mod` file to extract
    /// the module name, which typically represents the project.
    ///
    /// # Arguments
    ///
    /// * `go_mod` - Path to the `go.mod` file
    /// * `errors` - Shared error collection for reporting parsing issues
    ///
    /// # Returns
    ///
    /// - `Some(String)` containing the module name if successfully extracted
    /// - `None` if the name cannot be found or parsed
    ///
    /// # Parsing Strategy
    ///
    /// The method looks for the first line starting with `module ` and extracts
    /// the module path. For better display, it takes the last component of the path.
    fn extract_go_project_name(
        &self,
        go_mod: &Path,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        let content = self.read_file_content(go_mod, errors)?;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("module ") {
                let module_path = line.strip_prefix("module ")?.trim();

                // Take the last component of the module path for a cleaner name
                if let Some(name) = module_path.split('/').next_back() {
                    return Some(name.to_string());
                }

                return Some(module_path.to_string());
            }
        }

        None
    }

    /// Calculate the size of a directory recursively.
    ///
    /// This is a helper method used for Python projects to determine which
    /// cache directory is the largest and should be the primary cleanup target.
    fn calculate_directory_size(dir_path: &Path) -> std::io::Result<u64> {
        let mut total_size = 0;

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total_size += Self::calculate_directory_size(&path).unwrap_or(0);
            } else {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }
}
