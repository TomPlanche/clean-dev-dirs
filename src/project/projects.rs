//! Collection management and operations for development projects.
//!
//! This module provides the `Projects` struct which wraps a collection of
//! development projects and provides various operations on them, including
//! interactive selection, summary reporting, and parallel iteration support.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use humansize::{DECIMAL, format_size};
use rayon::prelude::*;

use crate::project::ProjectType;

use super::Project;

/// A collection of development projects with associated operations.
///
/// The `Projects` struct wraps a vector of `Project` instances and provides
/// higher-level operations such as interactive selection, summary reporting,
/// and parallel processing support. It serves as the main data structure
/// for managing collections of projects throughout the application.
pub(crate) struct Projects(Vec<Project>);

impl From<Vec<Project>> for Projects {
    /// Create a `Projects` collection from a vector of projects.
    ///
    /// This conversion allows easy creation of a `Projects` instance from
    /// any vector of `Project` objects, typically used when the scanner
    /// returns a collection of detected projects.
    ///
    /// # Arguments
    ///
    /// * `projects` - A vector of `Project` instances
    ///
    /// # Returns
    ///
    /// A new `Projects` collection containing the provided projects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::{Projects, Project};
    /// let project_vec = vec![/* project instances */];
    /// let projects: Projects = project_vec.into();
    /// ```
    fn from(projects: Vec<Project>) -> Self {
        Self(projects)
    }
}

impl IntoParallelIterator for Projects {
    type Iter = rayon::vec::IntoIter<Project>;
    type Item = Project;

    /// Enable parallel iteration with ownership transfer.
    ///
    /// This implementation allows the collection to be consumed and processed
    /// in parallel, transferring ownership of each project to the parallel
    /// processing context.
    ///
    /// # Returns
    ///
    /// A parallel iterator that takes ownership of the projects in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rayon::prelude::*;
    /// # use crate::Projects;
    /// let results: Vec<_> = projects.into_par_iter().map(|project| {
    ///     // Transform each project in parallel
    ///     process_project(project)
    /// }).collect();
    /// ```
    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

impl<'a> IntoParallelIterator for &'a Projects {
    type Iter = rayon::slice::Iter<'a, Project>;
    type Item = &'a Project;

    /// Enable parallel iteration over project references.
    ///
    /// This implementation allows the collection to be processed in parallel
    /// using Rayon's parallel iterators, which can significantly improve
    /// performance for operations that can be parallelized.
    ///
    /// # Returns
    ///
    /// A parallel iterator over references to the projects in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rayon::prelude::*;
    /// # use crate::Projects;
    /// projects.into_par_iter().for_each(|project| {
    ///     // Process each project in parallel
    ///     println!("Processing: {}", project.root_path.display());
    /// });
    /// ```
    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter()
    }
}

impl Projects {
    /// Calculate the total size of all build directories in the collection.
    ///
    /// This method sums up the sizes of all build directories (target/ or
    /// `node_modules`/) across all projects in the collection to provide a
    /// total estimate of reclaimable disk space.
    ///
    /// # Returns
    ///
    /// The total size in bytes of all build directories combined.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::Projects;
    /// let total_bytes = projects.get_total_size();
    /// println!("Total reclaimable space: {} bytes", total_bytes);
    /// ```
    pub(crate) fn get_total_size(&self) -> u64 {
        self.0.iter().map(|p| p.build_arts.size).sum()
    }

    /// Present an interactive selection interface for choosing projects to clean.
    ///
    /// This method displays a multi-select dialog that allows users to choose
    /// which projects they want to clean. Each project is shown with its type
    /// icon, path, and reclaimable space. All projects are selected by default.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Project>)` - The projects selected by the user
    /// - `Err(anyhow::Error)` - If the interactive dialog fails or is canceled
    ///
    /// # Interface Details
    ///
    /// - Uses a colorful theme for better visual appeal
    /// - Shows project type icons (🦀 for Rust, 📦 for Node.js)
    /// - Displays project paths and sizes in human-readable format
    /// - Allows toggling selections with space bar
    /// - Confirms selection with Enter key
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::Projects;
    /// # use anyhow::Result;
    /// let selected_projects = projects.interactive_selection()?;
    /// println!("User selected {} projects", selected_projects.len());
    /// ```
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The terminal doesn't support interactive input
    /// - The user cancels the dialog (Ctrl+C)
    /// - There are I/O errors with the terminal
    pub(crate) fn interactive_selection(&self) -> Result<Vec<Project>> {
        let items: Vec<String> = self
            .0
            .iter()
            .map(|p| {
                let icon = match p.kind {
                    ProjectType::Rust => "🦀",
                    ProjectType::Node => "📦",
                };
                format!(
                    "{icon} {} ({})",
                    p.root_path.display(),
                    format_size(p.build_arts.size, DECIMAL)
                )
            })
            .collect();

        let defaults = vec![true; self.0.len()];

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select projects to clean:")
            .items(&items)
            .defaults(&defaults)
            .interact()?;

        Ok(selections.into_iter().map(|i| self.0[i].clone()).collect())
    }

    /// Get the number of projects in the collection.
    ///
    /// # Returns
    ///
    /// The number of projects contained in this collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::Projects;
    /// println!("Found {} projects", projects.len());
    /// ```
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Print a detailed summary of the projects and their reclaimable space.
    ///
    /// This method analyzes the collection and prints statistics including:
    /// - Number and total size of Rust projects
    /// - Number and total size of Node.js projects  
    /// - Total reclaimable space across all projects
    ///
    /// The output is formatted with colors and emoji icons for better readability.
    ///
    /// # Arguments
    ///
    /// * `total_size` - The total size in bytes (usually from `get_total_size()`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::Projects;
    /// let total_size = projects.get_total_size();
    /// projects.print_summary(total_size);
    /// ```
    ///
    /// # Output Format
    ///
    /// ```text
    ///   🦀 5 Rust projects (2.3 GB)
    ///   📦 3 Node.js projects (1.7 GB)
    ///   💾 Total reclaimable space: 4.0 GB
    /// ```
    pub(crate) fn print_summary(&self, total_size: u64) {
        let mut rust_count = 0;
        let mut node_count = 0;
        let mut rust_size = 0u64;
        let mut node_size = 0u64;

        for project in &self.0 {
            match project.kind {
                ProjectType::Rust => {
                    rust_count += 1;
                    rust_size += project.build_arts.size;
                }
                ProjectType::Node => {
                    node_count += 1;
                    node_size += project.build_arts.size;
                }
            }
        }

        if rust_count > 0 {
            println!(
                "  🦀 {} Rust projects ({})",
                rust_count.to_string().bright_white(),
                format_size(rust_size, DECIMAL).bright_white()
            );
        }

        if node_count > 0 {
            println!(
                "  📦 {} Node.js projects ({})",
                node_count.to_string().bright_white(),
                format_size(node_size, DECIMAL).bright_white()
            );
        }

        println!(
            "  💾 Total reclaimable space: {}",
            format_size(total_size, DECIMAL).bright_green().bold()
        );
    }
}
