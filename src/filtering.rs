//! Project filtering functionality.
//!
//! This module provides functions for filtering projects based on various criteria
//! such as size and modification time.

use anyhow::Result;
use chrono::{DateTime, Local};
use rayon::prelude::*;
use std::fs;

use crate::config::FilterOptions;
use crate::project::Project;
use crate::utils::parse_size;

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
/// # Errors
///
/// This function can return errors if:
/// - The size string in `filter_opts.keep_size` cannot be parsed (invalid format)
/// - Size value overflow occurs during parsing
///
/// # Examples
///
/// ```no_run
/// # use clean_dev_dirs::{filtering::filter_projects, config::FilterOptions, project::Project};
/// # use anyhow::Result;
/// # fn example(projects: Vec<Project>) -> Result<()> {
/// let filter_opts = FilterOptions {
///     keep_size: "100MB".to_string(),
///     keep_days: 30,
/// };
/// let filtered = filter_projects(projects, &filter_opts)?;
/// # Ok(())
/// # }
/// ```
pub fn filter_projects(
    projects: Vec<Project>,
    filter_opts: &FilterOptions,
) -> Result<Vec<Project>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{BuildArtifacts, Project, ProjectType};
    use std::path::PathBuf;

    /// Helper function to create a test project
    fn create_test_project(
        kind: ProjectType,
        root_path: &str,
        build_path: &str,
        size: u64,
        name: Option<String>,
    ) -> Project {
        Project::new(
            kind,
            PathBuf::from(root_path),
            BuildArtifacts {
                path: PathBuf::from(build_path),
                size,
            },
            name,
        )
    }

    #[test]
    fn test_meets_size_criteria() {
        let project = create_test_project(
            ProjectType::Rust,
            "/test",
            "/test/target",
            1_000_000, // 1MB
            Some("test".to_string()),
        );

        assert!(meets_size_criteria(&project, 500_000)); // 0.5MB - should pass
        assert!(meets_size_criteria(&project, 1_000_000)); // Exactly 1MB - should pass
        assert!(!meets_size_criteria(&project, 2_000_000)); // 2MB - should fail
    }

    #[test]
    fn test_meets_time_criteria_disabled() {
        let project = create_test_project(
            ProjectType::Rust,
            "/test",
            "/test/target",
            1_000_000,
            Some("test".to_string()),
        );

        // When keep_days is 0, should always return true
        assert!(meets_time_criteria(&project, 0));
    }
}
