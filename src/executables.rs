//! Executable preservation logic.
//!
//! This module provides functionality to copy compiled executables out of
//! build directories before they are deleted during cleanup. This allows
//! users to retain usable binaries while still reclaiming build artifact space.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::project::{Project, ProjectType};

/// Extensions to exclude when looking for Rust executables.
const RUST_EXCLUDED_EXTENSIONS: &[&str] = &["d", "rmeta", "rlib", "a", "so", "dylib", "dll", "pdb"];

/// Check whether a file is an executable binary.
///
/// On Unix, this inspects the permission bits for the executable flag.
/// On Windows, this checks for the `.exe` file extension.
#[cfg(unix)]
fn is_executable(path: &Path, metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let _ = path; // unused on Unix – we rely on permission bits
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(windows)]
fn is_executable(path: &Path, _metadata: &fs::Metadata) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
}

/// A record of a single preserved executable file.
#[derive(Debug)]
pub struct PreservedExecutable {
    /// Original path inside the build directory
    pub source: PathBuf,
    /// Destination path where the file was copied
    pub destination: PathBuf,
}

/// Preserve compiled executables from a project's build directory.
///
/// Copies executable files to `<project_root>/bin/` before the build
/// directory is deleted. The behavior depends on the project type:
///
/// - **Rust**: copies executables from `target/release/` and `target/debug/`
/// - **Python**: copies `.whl` files from `dist/` and `.so`/`.pyd` extensions from `build/`
/// - **Node / Go**: no-op (their cleanable dirs are dependencies, not build outputs)
///
/// # Errors
///
/// Returns an error if creating destination directories or copying files fails.
pub fn preserve_executables(project: &Project) -> Result<Vec<PreservedExecutable>> {
    match project.kind {
        ProjectType::Rust => preserve_rust_executables(project),
        ProjectType::Python => preserve_python_executables(project),
        ProjectType::Node | ProjectType::Go => Ok(Vec::new()),
    }
}

/// Preserve Rust executables from `target/release/` and `target/debug/`.
fn preserve_rust_executables(project: &Project) -> Result<Vec<PreservedExecutable>> {
    let target_dir = &project.build_arts.path;
    let bin_dir = project.root_path.join("bin");
    let mut preserved = Vec::new();

    for profile in &["release", "debug"] {
        let profile_dir = target_dir.join(profile);
        if !profile_dir.is_dir() {
            continue;
        }

        let dest_dir = bin_dir.join(profile);
        let executables = find_rust_executables(&profile_dir)?;

        if executables.is_empty() {
            continue;
        }

        fs::create_dir_all(&dest_dir)
            .with_context(|| format!("Failed to create {}", dest_dir.display()))?;

        for exe_path in executables {
            let file_name = exe_path
                .file_name()
                .expect("executable path should have a file name");
            let dest_path = dest_dir.join(file_name);

            fs::copy(&exe_path, &dest_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    exe_path.display(),
                    dest_path.display()
                )
            })?;

            preserved.push(PreservedExecutable {
                source: exe_path,
                destination: dest_path,
            });
        }
    }

    Ok(preserved)
}

/// Find executable files in a Rust profile directory (e.g. `target/release/`).
///
/// Returns files that pass [`is_executable`] and are not build metadata
/// (excludes `.d`, `.rmeta`, `.rlib`, `.a`, `.so`, `.dylib`, `.dll`, `.pdb`
/// extensions).
fn find_rust_executables(profile_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut executables = Vec::new();

    let entries = fs::read_dir(profile_dir)
        .with_context(|| format!("Failed to read {}", profile_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Skip files with excluded extensions
        if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && RUST_EXCLUDED_EXTENSIONS.contains(&ext)
        {
            continue;
        }

        // Check if file is executable
        let metadata = path.metadata()?;
        if is_executable(&path, &metadata) {
            executables.push(path);
        }
    }

    Ok(executables)
}

/// Preserve Python build outputs: `.whl` from `dist/` and C extensions from `build/`.
fn preserve_python_executables(project: &Project) -> Result<Vec<PreservedExecutable>> {
    let root = &project.root_path;
    let bin_dir = root.join("bin");
    let mut preserved = Vec::new();

    // Copy .whl files from dist/
    let dist_dir = root.join("dist");
    if dist_dir.is_dir()
        && let Ok(entries) = fs::read_dir(&dist_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("whl") {
                fs::create_dir_all(&bin_dir)
                    .with_context(|| format!("Failed to create {}", bin_dir.display()))?;

                let file_name = path.file_name().expect("path should have a file name");
                let dest_path = bin_dir.join(file_name);

                fs::copy(&path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        path.display(),
                        dest_path.display()
                    )
                })?;

                preserved.push(PreservedExecutable {
                    source: path,
                    destination: dest_path,
                });
            }
        }
    }

    // Copy .so / .pyd C extensions from build/
    let build_dir = root.join("build");
    if build_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&build_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let is_extension = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext == "so" || ext == "pyd");

            if is_extension {
                fs::create_dir_all(&bin_dir)
                    .with_context(|| format!("Failed to create {}", bin_dir.display()))?;

                let file_name = path.file_name().expect("path should have a file name");
                let dest_path = bin_dir.join(file_name);

                fs::copy(path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        path.display(),
                        dest_path.display()
                    )
                })?;

                preserved.push(PreservedExecutable {
                    source: path.to_path_buf(),
                    destination: dest_path,
                });
            }
        }
    }

    Ok(preserved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::BuildArtifacts;
    use tempfile::TempDir;

    fn create_test_project(tmp: &TempDir, kind: ProjectType) -> Project {
        let root = tmp.path().to_path_buf();
        let build_dir = match kind {
            ProjectType::Rust => root.join("target"),
            ProjectType::Python => root.join("__pycache__"),
            ProjectType::Node => root.join("node_modules"),
            ProjectType::Go => root.join("vendor"),
        };

        fs::create_dir_all(&build_dir).unwrap();

        Project::new(
            kind,
            root,
            BuildArtifacts {
                path: build_dir,
                size: 0,
            },
            Some("test-project".to_string()),
        )
    }

    #[test]
    #[cfg(unix)]
    fn test_preserve_rust_executables_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Rust);

        // Create target/release/ with an executable and a metadata file
        let release_dir = tmp.path().join("target/release");
        fs::create_dir_all(&release_dir).unwrap();

        let exe_path = release_dir.join("my-binary");
        fs::write(&exe_path, b"fake binary").unwrap();
        fs::set_permissions(&exe_path, fs::Permissions::from_mode(0o755)).unwrap();

        let dep_file = release_dir.join("my-binary.d");
        fs::write(&dep_file, b"dep info").unwrap();

        let result = preserve_executables(&project).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].destination,
            tmp.path().join("bin/release/my-binary")
        );
        assert!(result[0].destination.exists());
    }

    #[test]
    #[cfg(windows)]
    fn test_preserve_rust_executables_windows() {
        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Rust);

        let release_dir = tmp.path().join("target/release");
        fs::create_dir_all(&release_dir).unwrap();

        // On Windows, executables have the .exe extension
        let exe_path = release_dir.join("my-binary.exe");
        fs::write(&exe_path, b"fake binary").unwrap();

        let dep_file = release_dir.join("my-binary.d");
        fs::write(&dep_file, b"dep info").unwrap();

        let result = preserve_executables(&project).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].destination,
            tmp.path().join("bin/release/my-binary.exe")
        );
        assert!(result[0].destination.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_preserve_rust_skips_non_executable_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Rust);

        let release_dir = tmp.path().join("target/release");
        fs::create_dir_all(&release_dir).unwrap();

        // Non-executable file (mode 0o644)
        let non_exe = release_dir.join("some-file");
        fs::write(&non_exe, b"not executable").unwrap();
        fs::set_permissions(&non_exe, fs::Permissions::from_mode(0o644)).unwrap();

        let result = preserve_executables(&project).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    #[cfg(windows)]
    fn test_preserve_rust_skips_non_executable_windows() {
        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Rust);

        let release_dir = tmp.path().join("target/release");
        fs::create_dir_all(&release_dir).unwrap();

        // On Windows, a file without .exe extension is not treated as executable
        let non_exe = release_dir.join("some-file.txt");
        fs::write(&non_exe, b"not executable").unwrap();

        let result = preserve_executables(&project).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_node_is_noop() {
        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Node);

        let result = preserve_executables(&project).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_go_is_noop() {
        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Go);

        let result = preserve_executables(&project).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_preserve_rust_no_profile_dirs() {
        let tmp = TempDir::new().unwrap();
        let project = create_test_project(&tmp, ProjectType::Rust);

        // target/ exists but no release/ or debug/ subdirs
        let result = preserve_executables(&project).unwrap();
        assert!(result.is_empty());
        assert!(!tmp.path().join("bin").exists());
    }
}
