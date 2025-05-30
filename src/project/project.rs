//! Core project data structures and types.
//!
//! This module defines the fundamental data structures used to represent
//! development projects and their build artifacts throughout the application.

use std::{
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

/// Enumeration of supported development project types.
///
/// This enum distinguishes between different types of development projects
/// that the tool can detect and clean. Each project type has its own
/// characteristic files and build directories.
#[derive(Clone, PartialEq)]
pub(crate) enum ProjectType {
    /// Rust project with Cargo.toml and target/ directory
    ///
    /// Rust projects are identified by the presence of both a `Cargo.toml`
    /// file and a `target/` directory in the same location.
    Rust,
    
    /// Node.js project with package.json and node_modules/ directory
    ///
    /// Node.js projects are identified by the presence of both a `package.json`
    /// file and a `node_modules/` directory in the same location.
    Node,
}

/// Information about build artifacts that can be cleaned.
///
/// This struct contains metadata about the build directory or artifacts
/// that are candidates for cleanup, including their location and total size.
#[derive(Clone)]
pub(crate) struct BuildArtifacts {
    /// Path to the build directory (target/ or node_modules/)
    ///
    /// This is the directory that will be deleted during cleanup operations.
    /// For Rust projects, this points to the `target/` directory.
    /// For Node.js projects, this points to the `node_modules/` directory.
    pub(crate) path: PathBuf,
    
    /// Total size of the build directory in bytes
    ///
    /// This value is calculated by recursively summing the sizes of all files
    /// within the build directory. It's used for filtering and reporting purposes.
    pub(crate) size: u64,
}

/// Representation of a development project with cleanable build artifacts.
///
/// This struct encapsulates all information about a development project,
/// including its type, location, build artifacts, and metadata extracted
/// from project configuration files.
#[derive(Clone)]
pub(crate) struct Project {
    /// Type of the project (Rust or Node.js)
    pub(crate) kind: ProjectType,

    /// The root directory of the project where the configuration file is located
    ///
    /// For Rust projects, this is the directory containing `Cargo.toml`.
    /// For Node.js projects, this is the directory containing `package.json`.
    pub(crate) root_path: PathBuf,

    /// The build directory to be cleaned and its metadata
    ///
    /// Contains information about the `target/` or `node_modules/` directory
    /// that is a candidate for cleanup, including its path and total size.
    pub(crate) build_arts: BuildArtifacts,

    /// Name of the project extracted from configuration files
    ///
    /// For Rust projects, this is extracted from the `name` field in `Cargo.toml`.
    /// For Node.js projects, this is extracted from the `name` field in `package.json`.
    /// May be `None` if the name cannot be determined or parsed.
    pub(crate) name: Option<String>,
}

impl Project {
    /// Create a new project instance.
    ///
    /// This constructor creates a new `Project` with the specified parameters.
    /// It's typically used by the scanner when a valid development project
    /// is detected in the file system.
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of project (Rust or Node.js)
    /// * `root_path` - Path to the project's root directory
    /// * `build_arts` - Information about the build artifacts to be cleaned
    /// * `name` - Optional project name extracted from configuration files
    ///
    /// # Returns
    ///
    /// A new `Project` instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use crate::project::{Project, ProjectType, BuildArtifacts};
    /// let build_arts = BuildArtifacts {
    ///     path: PathBuf::from("/path/to/project/target"),
    ///     size: 1024,
    /// };
    /// 
    /// let project = Project::new(
    ///     ProjectType::Rust,
    ///     PathBuf::from("/path/to/project"),
    ///     build_arts,
    ///     Some("my-project".to_string()),
    /// );
    /// ```
    pub fn new(
        kind: ProjectType,
        root_path: PathBuf,
        build_arts: BuildArtifacts,
        name: Option<String>,
    ) -> Self {
        Self {
            kind,
            root_path,
            build_arts,
            name,
        }
    }
}

impl Display for Project {
    /// Format the project for display with the appropriate emoji and name.
    ///
    /// This implementation provides a human-readable representation of the project
    /// that includes:
    /// - An emoji indicator based on the project type (🦀 for Rust, 📦 for Node.js)
    /// - The project name if available, otherwise just the path
    /// - The project's root path
    ///
    /// # Examples
    ///
    /// - `🦀 my-rust-project (/path/to/project)`
    /// - `📦 my-node-app (/path/to/app)`
    /// - `🦀 /path/to/unnamed/project` (when no name is available)
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let icon = match self.kind {
            ProjectType::Rust => "🦀",
            ProjectType::Node => "📦",
        };

        if let Some(name) = &self.name {
            write!(f, "{icon} {name} ({})", self.root_path.display())
        } else {
            write!(f, "{icon} {}", self.root_path.display())
        }
    }
}
