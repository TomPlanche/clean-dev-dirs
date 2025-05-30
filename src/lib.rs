//! # clean-dev-dirs
//!
//! A fast and efficient CLI tool for recursively cleaning Rust `target/` and Node.js `node_modules/`
//! directories to reclaim disk space.
//!
//! This library provides the core functionality for the clean-dev-dirs CLI tool,
//! allowing for scanning, filtering, and cleaning development project build artifacts.

pub(crate) mod cleaner;
pub mod cli;
pub mod project;
pub mod scanner;
