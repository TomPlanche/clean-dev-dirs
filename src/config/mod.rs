//! Configuration types and options for the application.
//!
//! This module contains all configuration structures used throughout the application,
//! including filtering, scanning, and execution options.

pub mod execution;
pub mod filter;
pub mod scan;

pub use execution::ExecutionOptions;
pub use filter::{FilterOptions, ProjectFilter};
pub use scan::ScanOptions;
