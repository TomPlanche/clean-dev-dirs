//! Project detection and management functionality.
//!
//! This module contains the core data structures and logic for representing
//! and managing development projects. It provides types for individual projects,
//! collections of projects, and the operations that can be performed on them.
//!
//! ## Main Parts
//!
//! - [`Project`] - Represents an individual development project with build artifacts
//! - [`Projects`] - A collection of projects with batch operations
//! - [`ProjectType`] - Enumeration of supported project types (Rust, Node.js)
//! - [`BuildArtifacts`] - Information about build directories and their sizes

#[allow(clippy::module_inception)]
pub(crate) mod project;
pub(crate) mod projects;

pub(crate) use project::{BuildArtifacts, Project, ProjectType};
pub(crate) use projects::Projects;
