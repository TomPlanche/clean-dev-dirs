//! Filtering configuration for project selection.
//!
//! This module defines the filtering options and project type filters used to
//! determine which projects should be scanned and cleaned.

use clap::ValueEnum;

/// Enumeration of supported project type filters.
///
/// This enum is used to restrict scanning and cleaning to specific types of
/// development projects.
#[derive(Clone, Copy, PartialEq, Debug, ValueEnum, Default)]
pub enum ProjectFilter {
    /// Include all supported project types (Rust, Node.js, Python, Go)
    #[default]
    All,

    /// Include only Rust projects (Cargo.toml + target/)
    Rust,

    /// Include only Node.js projects (package.json + `node_modules`/)
    Node,

    /// Include only Python projects (Python config files + cache dirs)
    Python,

    /// Include only Go projects (go.mod + vendor/)
    Go,
}

/// Configuration for project filtering criteria.
///
/// This struct contains the filtering options used to determine which projects
/// should be considered for cleanup based on size and modification time.
#[derive(Clone)]
pub struct FilterOptions {
    /// Minimum size threshold for build directories
    pub keep_size: String,

    /// Minimum age in days for projects to be considered
    pub keep_days: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_filter_equality() {
        assert_eq!(ProjectFilter::All, ProjectFilter::All);
        assert_eq!(ProjectFilter::Rust, ProjectFilter::Rust);
        assert_eq!(ProjectFilter::Node, ProjectFilter::Node);
        assert_eq!(ProjectFilter::Python, ProjectFilter::Python);
        assert_eq!(ProjectFilter::Go, ProjectFilter::Go);

        assert_ne!(ProjectFilter::All, ProjectFilter::Rust);
        assert_ne!(ProjectFilter::Rust, ProjectFilter::Node);
        assert_ne!(ProjectFilter::Node, ProjectFilter::Python);
        assert_ne!(ProjectFilter::Python, ProjectFilter::Go);
    }

    #[test]
    fn test_project_filter_copy() {
        let original = ProjectFilter::Rust;
        let copied = original;

        assert_eq!(original, copied);
    }

    #[test]
    fn test_project_filter_default() {
        let default_filter = ProjectFilter::default();
        assert_eq!(default_filter, ProjectFilter::All);
    }

    #[test]
    fn test_filter_options_creation() {
        let filter_opts = FilterOptions {
            keep_size: "100MB".to_string(),
            keep_days: 30,
        };

        assert_eq!(filter_opts.keep_size, "100MB");
        assert_eq!(filter_opts.keep_days, 30);
    }

    #[test]
    fn test_filter_options_clone() {
        let original = FilterOptions {
            keep_size: "100MB".to_string(),
            keep_days: 30,
        };
        let cloned = original.clone();

        assert_eq!(original.keep_size, cloned.keep_size);
        assert_eq!(original.keep_days, cloned.keep_days);
    }
}
