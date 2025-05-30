use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::{Value, from_str};
use walkdir::{DirEntry, WalkDir};

use crate::{
    cli::ProjectFilter,
    project::{BuildArtifacts, Project, ProjectType},
};

#[derive(Debug)] // TODO: Remove Debug
pub(crate) struct Scanner {
    verbose: bool,
    skip_dirs: HashSet<PathBuf>,
    project_filter: ProjectFilter,
}

impl Scanner {
    pub(crate) fn new(
        verbose: bool,
        skip_dirs: Vec<PathBuf>,
        project_filter: ProjectFilter,
    ) -> Self {
        Self {
            verbose,
            skip_dirs: skip_dirs.into_iter().collect(),
            project_filter,
        }
    }

    pub(crate) fn scan_directory(&self, root: &Path) -> Vec<Project> {
        let errors = Arc::new(Mutex::new(Vec::<String>::new()));

        // Create progress bar
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
        if self.verbose {
            let errors = errors.lock().unwrap();
            for error in errors.iter() {
                eprintln!("{}", error.red());
            }
        }

        projects_with_sizes
    }

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
                    if self.verbose {
                        eprintln!("Warning: {e}");
                    }
                }
            }
        }

        total_size
    }

    fn detect_project(
        &self,
        entry: &DirEntry,
        errors: &Arc<Mutex<Vec<String>>>,
    ) -> Option<Project> {
        let path = entry.path();

        if !entry.file_type().is_dir() {
            return None;
        }

        // Check for Rust project
        if matches!(
            self.project_filter,
            ProjectFilter::All | ProjectFilter::RustOnly
        ) {
            if let Some(project) = self.detect_rust_project(path, errors) {
                return Some(project);
            }
        }

        // Check for Node.js project
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
                if self.verbose {
                    let mut errors = errors.lock().unwrap();
                    errors.push(format!("Error reading {}: {e}", cargo_toml.display()));
                }
                None
            }
        }
    }

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
                    if self.verbose {
                        let mut errors = errors.lock().unwrap();
                        errors.push(format!("Error parsing {}: {e}", package_json.display()));
                    }
                    None
                }
            },
            Err(e) => {
                if self.verbose {
                    let mut errors = errors.lock().unwrap();
                    errors.push(format!("Error reading {}: {e}", package_json.display()));
                }
                None
            }
        }
    }

    fn should_scan_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        // Skip directories in the skip list
        if self.skip_dirs.iter().any(|skip| path.starts_with(skip)) {
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
            "node_modules",
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
