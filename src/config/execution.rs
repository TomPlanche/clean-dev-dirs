//! Execution configuration for cleanup operations.
//!
//! This module defines the options that control how cleanup operations are executed,
//! including dry-run mode and interactive selection.

/// Configuration for cleanup execution behavior.
///
/// This struct provides a simplified interface to execution-related options,
/// controlling how the cleanup process runs.
#[derive(Clone)]
pub struct ExecutionOptions {
    /// Whether to run in dry-run mode (no actual deletion)
    pub dry_run: bool,

    /// Whether to use interactive project selection
    pub interactive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_options_creation() {
        let exec_opts = ExecutionOptions {
            dry_run: true,
            interactive: false,
        };

        assert!(exec_opts.dry_run);
        assert!(!exec_opts.interactive);
    }

    #[test]
    fn test_execution_options_clone() {
        let original = ExecutionOptions {
            dry_run: true,
            interactive: false,
        };
        let cloned = original.clone();

        assert_eq!(original.dry_run, cloned.dry_run);
        assert_eq!(original.interactive, cloned.interactive);
    }
}
