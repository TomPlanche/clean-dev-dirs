//! Integration tests for clean-dev-dirs
//!
//! These tests create temporary file structures to test the real functionality
//! of the scanner and other components with actual filesystem operations.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use clean_dev_dirs::cli::{ProjectFilter, ScanOptions};
use clean_dev_dirs::project::{BuildArtifacts, ProjectType};
use clean_dev_dirs::scanner::Scanner;

/// Helper function to create a temporary directory structure for testing
fn create_test_directory() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Helper function to create a file with specified content
fn create_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directories");
    }
    fs::write(path, content).expect("Failed to write file");
}

/// Helper function to create a directory
fn create_dir(path: &Path) {
    fs::create_dir_all(path).expect("Failed to create directory");
}

/// Create a mock Rust project with Cargo.toml and target/ directory
fn create_rust_project(base_path: &Path, project_name: &str) -> PathBuf {
    let project_path = base_path.join(project_name);

    // Create Cargo.toml
    let cargo_toml_content = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    );
    create_file(&project_path.join("Cargo.toml"), &cargo_toml_content);

    // Create target directory with some files
    let target_path = project_path.join("target");
    create_dir(&target_path);
    create_file(
        &target_path.join("debug").join("build.log"),
        "Build log content",
    );
    create_file(
        &target_path.join("release").join("binary"),
        "Binary content",
    );

    project_path
}

/// Create a mock Node.js project with package.json and `node_modules`/ directory
fn create_node_project(base_path: &Path, project_name: &str) -> PathBuf {
    let project_path = base_path.join(project_name);

    // Create package.json
    let package_json_content = format!(
        r#"{{
  "name": "{project_name}",
  "version": "1.0.0",
  "description": "Test Node.js project",
  "main": "index.js",
  "dependencies": {{
    "express": "^4.18.0"
  }}
}}"#
    );
    create_file(&project_path.join("package.json"), &package_json_content);

    // Create node_modules directory with some files
    let node_modules_path = project_path.join("node_modules");
    create_dir(&node_modules_path);
    create_file(
        &node_modules_path.join("express").join("package.json"),
        "{}",
    );
    create_file(
        &node_modules_path.join(".bin").join("express"),
        "#!/bin/bash",
    );

    project_path
}

/// Create a mock Python project with requirements.txt and __pycache__/ directory
fn create_python_project(base_path: &Path, project_name: &str) -> PathBuf {
    let project_path = base_path.join(project_name);

    // Create requirements.txt
    create_file(
        &project_path.join("requirements.txt"),
        "requests==2.28.0\nflask==2.3.0\n",
    );

    // Create __pycache__ directory with some files
    let pycache_path = project_path.join("__pycache__");
    create_dir(&pycache_path);
    create_file(&pycache_path.join("main.cpython-39.pyc"), "Python bytecode");
    create_file(
        &pycache_path.join("utils.cpython-39.pyc"),
        "Python bytecode",
    );

    project_path
}

/// Create a mock Go project with go.mod and vendor/ directory
fn create_go_project(base_path: &Path, project_name: &str) -> PathBuf {
    let project_path = base_path.join(project_name);

    // Create go.mod
    let go_mod_content = format!(
        r"module {project_name}

go 1.19

require (
    github.com/gin-gonic/gin v1.9.0
)
"
    );
    create_file(&project_path.join("go.mod"), &go_mod_content);

    // Create vendor directory with some files
    let vendor_path = project_path.join("vendor");
    create_dir(&vendor_path);
    create_file(
        &vendor_path
            .join("github.com")
            .join("gin-gonic")
            .join("gin")
            .join("gin.go"),
        "package gin",
    );

    project_path
}

#[test]
fn test_scanner_finds_rust_projects() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create test projects
    create_rust_project(base_path, "rust-project-1");
    create_rust_project(base_path, "rust-project-2");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::RustOnly);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 2);

    for project in &projects {
        assert_eq!(project.kind, ProjectType::Rust);
        assert!(project.name.is_some());
        assert!(project.build_arts.path.ends_with("target"));
        assert!(project.build_arts.size > 0);
    }
}

#[test]
fn test_scanner_finds_node_projects() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create test projects
    create_node_project(base_path, "node-app-1");
    create_node_project(base_path, "node-app-2");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::NodeOnly);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 2);

    for project in &projects {
        assert_eq!(project.kind, ProjectType::Node);
        assert!(project.name.is_some());
        assert!(project.build_arts.path.ends_with("node_modules"));
        assert!(project.build_arts.size > 0);
    }
}

#[test]
fn test_scanner_finds_python_projects() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create test projects
    create_python_project(base_path, "python-app-1");
    create_python_project(base_path, "python-app-2");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::PythonOnly);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 2);

    for project in &projects {
        assert_eq!(project.kind, ProjectType::Python);
        assert!(project.build_arts.path.ends_with("__pycache__"));
        assert!(project.build_arts.size > 0);
    }
}

#[test]
fn test_scanner_finds_go_projects() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create test projects
    create_go_project(base_path, "go-service-1");
    create_go_project(base_path, "go-service-2");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::GoOnly);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 2);

    for project in &projects {
        assert_eq!(project.kind, ProjectType::Go);
        assert!(project.name.is_some());
        assert!(project.build_arts.path.ends_with("vendor"));
        assert!(project.build_arts.size > 0);
    }
}

#[test]
fn test_scanner_finds_all_project_types() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create one of each project type
    create_rust_project(base_path, "rust-project");
    create_node_project(base_path, "node-project");
    create_python_project(base_path, "python-project");
    create_go_project(base_path, "go-project");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::All);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 4);

    let mut found_types = vec![];
    for project in &projects {
        found_types.push(project.kind.clone());
    }

    assert!(found_types.contains(&ProjectType::Rust));
    assert!(found_types.contains(&ProjectType::Node));
    assert!(found_types.contains(&ProjectType::Python));
    assert!(found_types.contains(&ProjectType::Go));
}

#[test]
fn test_scanner_skips_directories() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create projects in various subdirectories
    create_rust_project(base_path, "rust-project");
    create_rust_project(&base_path.join("target"), "nested-rust-project");
    create_rust_project(&base_path.join("skip-me"), "skipped-rust-project");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![PathBuf::from("skip-me"), PathBuf::from("target")],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::RustOnly);
    let projects = scanner.scan_directory(base_path);

    // Should only find the top-level project, not the ones in skipped directories
    assert_eq!(projects.len(), 1);
    assert!(projects[0].root_path.ends_with("rust-project"));
}

#[test]
fn test_scanner_calculates_build_directory_sizes() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    let project_path = create_rust_project(base_path, "rust-project");
    let target_path = project_path.join("target");

    // Add more files with known sizes
    create_file(&target_path.join("large-file.bin"), &"x".repeat(1000));
    create_file(&target_path.join("small-file.txt"), "small");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::RustOnly);
    let projects = scanner.scan_directory(base_path);

    assert_eq!(projects.len(), 1);

    let project = &projects[0];
    assert!(project.build_arts.size > 1000); // Should include our large file
}

#[test]
fn test_scanner_handles_empty_directories() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create a project structure but with empty build directories
    let project_path = base_path.join("empty-rust-project");
    create_file(
        &project_path.join("Cargo.toml"),
        "[package]\nname = \"empty\"\nversion = \"0.1.0\"",
    );
    create_dir(&project_path.join("target")); // Empty target directory

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::RustOnly);
    let projects = scanner.scan_directory(base_path);

    // Empty target directories should not be included (size = 0)
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_scanner_handles_missing_build_directories() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create project configuration but no build directory
    let project_path = base_path.join("no-target-project");
    create_file(
        &project_path.join("Cargo.toml"),
        "[package]\nname = \"no-target\"\nversion = \"0.1.0\"",
    );
    // No target directory created

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::RustOnly);
    let projects = scanner.scan_directory(base_path);

    // Projects without build directories should not be found
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_scanner_nested_projects() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create nested project structure
    create_rust_project(base_path, "parent-project");
    create_rust_project(&base_path.join("parent-project"), "child-project");
    create_node_project(&base_path.join("parent-project").join("frontend"), "ui-app");

    let scan_options = ScanOptions {
        verbose: false,
        threads: 1,
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::All);
    let projects = scanner.scan_directory(base_path);

    // Should find all 3 projects (2 Rust + 1 Node.js)
    assert_eq!(projects.len(), 3);

    let rust_projects: Vec<_> = projects
        .iter()
        .filter(|p| p.kind == ProjectType::Rust)
        .collect();
    let node_projects: Vec<_> = projects
        .iter()
        .filter(|p| p.kind == ProjectType::Node)
        .collect();

    assert_eq!(rust_projects.len(), 2);
    assert_eq!(node_projects.len(), 1);
}

#[test]
fn test_scanner_with_multiple_threads() {
    let temp_dir = create_test_directory();
    let base_path = temp_dir.path();

    // Create multiple projects
    for i in 0..10 {
        create_rust_project(base_path, &format!("rust-project-{i}"));
        create_node_project(base_path, &format!("node-project-{i}"));
    }

    let scan_options = ScanOptions {
        verbose: false,
        threads: 4, // Use multiple threads
        skip: vec![],
    };

    let scanner = Scanner::new(scan_options, ProjectFilter::All);
    let projects = scanner.scan_directory(base_path);

    // Should find all 20 projects (10 Rust + 10 Node.js)
    assert_eq!(projects.len(), 20);
}

#[test]
fn test_build_artifacts_structure() {
    let temp_dir = create_test_directory();
    let project_path = create_rust_project(temp_dir.path(), "test-project");
    let target_path = project_path.join("target");

    let artifacts = BuildArtifacts {
        path: target_path.clone(),
        size: 12345,
    };

    assert_eq!(artifacts.path, target_path);
    assert_eq!(artifacts.size, 12345);

    // Test cloning
    let cloned = artifacts.clone();
    assert_eq!(artifacts.path, cloned.path);
    assert_eq!(artifacts.size, cloned.size);
}

#[test]
fn test_project_types_comprehensive() {
    // Test all project type variants
    assert_eq!(ProjectType::Rust, ProjectType::Rust);
    assert_eq!(ProjectType::Node, ProjectType::Node);
    assert_eq!(ProjectType::Python, ProjectType::Python);
    assert_eq!(ProjectType::Go, ProjectType::Go);

    // Test cloning
    let rust_type = ProjectType::Rust;
    let cloned_type = rust_type.clone();
    assert_eq!(rust_type, cloned_type);
}
