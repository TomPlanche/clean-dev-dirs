//! # clean-dev-dirs
//!
//! A fast and efficient CLI tool for recursively cleaning Rust `target/` and Node.js `node_modules/`
//! directories to reclaim disk space.
//!
//! This library provides the core functionality for the clean-dev-dirs CLI tool,
//! allowing for scanning, filtering, and cleaning development project build artifacts.

pub mod cleaner;
pub mod config;
pub mod filtering;
pub mod project;
pub mod scanner;
pub mod utils;

// Re-export commonly used types for convenience
pub use cleaner::Cleaner;
pub use config::{ExecutionOptions, FilterOptions, ProjectFilter, ScanOptions};
pub use filtering::filter_projects;
pub use project::{BuildArtifacts, Project, ProjectType, Projects};
pub use scanner::Scanner;
pub use utils::parse_size;
