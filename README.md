# 🧹 clean-dev-dirs

> A fast and efficient CLI tool for recursively cleaning Rust `target/`, Node.js `node_modules/`, Python cache, and Go `vendor/` directories to reclaim disk space.

<p align="center">
  <a href="https://crates.io/crates/clean-dev-dirs"><img src="https://img.shields.io/crates/v/clean-dev-dirs.svg" alt="Crates.io Version"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=alert_status" alt="SonarCloud Status"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=sqale_rating" alt="SonarCloud SQALE Rating"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=security_rating" alt="SonarCloud Security Rating"></a>
  <a href="https://github.com/TomPlanche/clean-dev-dirs/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clean-dev-dirs" alt="License"></a>
</p>

## Quick Start

```bash
# Install from crates.io
cargo install clean-dev-dirs

# Clean all development directories in current directory
clean-dev-dirs

# Preview what would be cleaned (dry run)
clean-dev-dirs --dry-run

# Interactive mode - choose which projects to clean
clean-dev-dirs --interactive
```

## Features

- **Multi-language support**: Clean Rust (`target/`), Node.js (`node_modules/`), Python (cache dirs), and Go (`vendor/`) build artifacts
- **Parallel scanning**: Lightning-fast directory traversal using multithreading
- **Smart filtering**: Filter by project size, modification time, and project type
- **Interactive mode**: Choose which projects to clean with an intuitive interface
- **Dry-run mode**: Preview what would be cleaned without actually deleting anything
- **Progress indicators**: Real-time feedback during scanning and cleaning operations
- **Detailed statistics**: See total space that can be reclaimed before cleaning
- **Flexible configuration**: Combine multiple filters and options for precise control

## Inspiration

This project is inspired by [cargo-clean-all](https://github.com/dnlmlr/cargo-clean-all), a Rust-specific tool for cleaning cargo projects. We've improved upon the original concept with:

- **Multi-language support**: Extended beyond Rust to support Node.js, Python, and Go projects
- **Parallel scanning**: Significantly faster directory traversal using multithreading
- **Enhanced filtering**: More granular control over what gets cleaned
- **Cleaner code architecture**: Well-structured, modular codebase for better maintainability

## Installation

### From crates.io (Recommended)

```bash
cargo install clean-dev-dirs
```

### From Source

```bash
git clone https://github.com/TomPlanche/clean-dev-dirs.git
cd clean-dev-dirs
cargo install --path .
```

### Requirements

- Rust 2021 edition or later
- Cargo package manager

## Usage

### Basic Usage

```bash
# Clean all development directories in the current directory
clean-dev-dirs

# Clean a specific directory
clean-dev-dirs ~/Projects

# Preview what would be cleaned (dry run)
clean-dev-dirs --dry-run

# Interactive mode - choose which projects to clean
clean-dev-dirs --interactive
```

### Project Type Filtering

```bash
# Clean only Rust projects
clean-dev-dirs --project-type rust
# or use short flag
clean-dev-dirs -p rust

# Clean only Node.js projects
clean-dev-dirs -p node

# Clean only Python projects
clean-dev-dirs -p python

# Clean only Go projects
clean-dev-dirs -p go

# Clean all project types (default)
clean-dev-dirs -p all
```

### Size and Time Filtering

```bash
# Only clean projects with build dirs larger than 100MB
clean-dev-dirs --keep-size 100MB

# Only clean projects not modified in the last 30 days
clean-dev-dirs --keep-days 30

# Combine size and time filters
clean-dev-dirs --keep-size 50MB --keep-days 7
```

### Advanced Options

```bash
# Use 8 threads for faster scanning
clean-dev-dirs --threads 8

# Show verbose output including scan errors
clean-dev-dirs --verbose

# Skip specific directories during scanning
clean-dev-dirs --skip node_modules --skip .git

# Non-interactive mode (auto-confirm)
clean-dev-dirs --yes

# Combine multiple options
clean-dev-dirs ~/Projects -p rust --keep-size 100MB --keep-days 30 --dry-run
```

### Common Use Cases

**1. Clean old Rust projects:**
```bash
clean-dev-dirs ~/Projects -p rust --keep-days 90
```

**2. Preview large Python cache directories:**
```bash
clean-dev-dirs ~/workspace -p python --keep-size 50MB --dry-run
```

**3. Interactive cleaning of all Node.js projects:**
```bash
clean-dev-dirs ~/dev -p node --interactive
```

**4. Quick cleanup with confirmation:**
```bash
clean-dev-dirs ~/code --keep-size 100MB --keep-days 60
```

**5. Fast scan with multiple threads:**
```bash
clean-dev-dirs /large/directory --threads 16 --verbose
```

## Command Reference

### Main Arguments

| Argument | Description |
|----------|-------------|
| `[DIR]` | Directory to search for projects (default: current directory) |

### Project Type Filter

| Option | Short | Values | Description |
|--------|-------|--------|-------------|
| `--project-type` | `-p` | `all`, `rust`, `node`, `python`, `go` | Filter by project type (default: `all`) |

### Filtering Options

| Option | Short | Description |
|--------|-------|-------------|
| `--keep-size` | `-s` | Ignore projects with build dir smaller than specified size |
| `--keep-days` | `-d` | Ignore projects modified in the last N days |

### Execution Options

| Option | Short | Description |
|--------|-------|-------------|
| `--yes` | `-y` | Don't ask for confirmation; clean all detected projects |
| `--dry-run` | | List cleanable projects without actually cleaning |
| `--interactive` | `-i` | Use interactive project selection |

### Scanning Options

| Option | Short | Description |
|--------|-------|-------------|
| `--threads` | `-t` | Number of threads for directory scanning (default: CPU cores) |
| `--verbose` | `-v` | Show access errors during scanning |
| `--skip` | | Directories to skip during scanning (can be specified multiple times) |

## Size Formats

The `--keep-size` option supports various size formats:

| Format | Example | Description |
|--------|---------|-------------|
| **Decimal** | `100KB`, `1.5MB`, `2GB` | Base 1000 |
| **Binary** | `100KiB`, `1.5MiB`, `2GiB` | Base 1024 |
| **Bytes** | `1000000` | Raw byte count |

### Examples:
```bash
clean-dev-dirs --keep-size 100KB    # 100 kilobytes
clean-dev-dirs --keep-size 1.5MB    # 1.5 megabytes
clean-dev-dirs --keep-size 2GiB     # 2 gibibytes
clean-dev-dirs --keep-size 500000   # 500,000 bytes
```

## Project Detection

The tool automatically detects development projects by looking for characteristic files and directories:

### Rust Projects
- **Detection criteria**: Both `Cargo.toml` and `target/` directory must exist
- **Cleans**: `target/` directory
- **Name extraction**: From `[package] name` in `Cargo.toml`

### Node.js Projects
- **Detection criteria**: Both `package.json` and `node_modules/` directory must exist
- **Cleans**: `node_modules/` directory
- **Name extraction**: From `name` field in `package.json`

### Python Projects
- **Detection criteria**:
  - At least one config file: `requirements.txt`, `setup.py`, `pyproject.toml`, `setup.cfg`, `Pipfile`, `pipenv.lock`, `poetry.lock`
  - At least one cache/build directory exists
- **Cleans**: The largest cache/build directory among:
  - `__pycache__`
  - `.pytest_cache`
  - `venv` / `.venv`
  - `build` / `dist`
  - `.eggs` / `.tox` / `.coverage`
- **Name extraction**: From `pyproject.toml` (project name or tool.poetry name) or `setup.py`

### Go Projects
- **Detection criteria**: Both `go.mod` and `vendor/` directory must exist
- **Cleans**: `vendor/` directory
- **Name extraction**: From module path in `go.mod`

## Safety Features

- **Dry-run mode**: Preview all operations before execution with `--dry-run`
- **Interactive confirmation**: Manually select projects to clean with `--interactive`
- **Intelligent filtering**: Skip recently modified or small projects with `--keep-days` and `--keep-size`
- **Error handling**: Graceful handling of permission errors and inaccessible files
- **Read-only scanning**: Never modifies files during the scanning phase
- **Clear output**: Color-coded, human-readable output with project types and sizes

## Output

The tool provides beautiful, colored output including:

| Icon | Project Type |
|------|--------------|
| 🦀 | Rust projects |
| 📦 | Node.js projects |
| 🐍 | Python projects |
| 🐹 | Go projects |

### Sample Output

```
Found 15 projects

📊 Found projects:

🦀 my-rust-app (/home/user/projects/rust-app)
   Size: 2.3 GB

📦 web-frontend (/home/user/projects/web-app)
   Size: 856 MB

🐍 ml-project (/home/user/projects/python-ml)
   Size: 1.2 GB

Total space that can be reclaimed: 4.4 GB
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding Language Support

Want to add support for a new programming language? Here's how to extend `clean-dev-dirs`:

#### 1. **Update Project Types**

First, add your language to the `ProjectType` enum in `src/project/project.rs`:

```rust
#[derive(Clone, PartialEq, Debug)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    YourLanguage, // Add your language here
}
```

Don't forget to update the `Display` implementation to include an appropriate emoji and name.

#### 2. **Add CLI Filter Option**

Update `src/config/filter.rs` to add your language to the `ProjectFilter` enum:

```rust
#[derive(Clone, Copy, PartialEq, Debug, ValueEnum, Default)]
pub enum ProjectFilter {
    All,
    Rust,
    Node,
    Python,
    Go,
    YourLanguage, // Add here
}
```

#### 3. **Implement Project Detection**

Add detection logic in `src/scanner.rs` by implementing:

- **Detection method**: `detect_your_language_project()` - identifies projects by looking for characteristic files
- **Name extraction**: `extract_your_language_project_name()` - parses project configuration files to get the name
- **Integration**: Update `detect_project()` to call your detection method

**Example detection criteria:**
```rust
fn detect_your_language_project(&self, path: &Path, errors: &Arc<Mutex<Vec<String>>>) -> Option<Project> {
    let config_file = path.join("your_config.conf");  // Language-specific config file
    let build_dir = path.join("build");               // Build/cache directory to clean

    if config_file.exists() && build_dir.exists() {
        let name = self.extract_your_language_project_name(&config_file, errors);

        let build_arts = BuildArtifacts {
            path: build_dir,
            size: 0, // Will be calculated later
        };

        return Some(Project::new(
            ProjectType::YourLanguage,
            path.to_path_buf(),
            build_arts,
            name,
        ));
    }

    None
}
```

#### 4. **Update Directory Exclusions**

Add any language-specific directories that should be skipped during scanning to the `should_scan_entry()` method in `src/scanner.rs`.

#### 5. **Update Documentation**

- Add your language to the "Project Detection" section in this README
- Update the CLI help text descriptions
- Add examples in the usage section

#### 6. **Testing Considerations**

Consider these when testing your implementation:

- **Multiple config files**: Some languages have different project file formats
- **Build directory variations**: Different build tools may use different directory names
- **Name extraction edge cases**: Handle malformed or missing project names gracefully
- **Performance**: Ensure detection doesn't significantly slow down scanning

#### 7. **Example Languages to Add**

Some languages that would be great additions:

- **C/C++**: Look for `CMakeLists.txt`/`Makefile` + `build/` or `cmake-build-*/`
- **Java**: Look for `pom.xml`/`build.gradle` + `target/` or `build/`
- **C#**: Look for `*.csproj`/`*.sln` + `bin/`/`obj/`
- **PHP**: Look for `composer.json` + `vendor/`
- **Ruby**: Look for `Gemfile` + `vendor/bundle/`
- **Swift**: Look for `Package.swift` + `.build/`

#### 8. **Pull Request Guidelines**

When submitting your language support:

1. **Test thoroughly**: Verify detection works with real projects
2. **Add examples**: Include sample project structures in your PR description
3. **Update help text**: Ensure all user-facing text is clear and consistent
4. **Follow patterns**: Use the same patterns as existing language implementations
5. **Consider edge cases**: Handle projects with unusual structures gracefully

## License

This project is dual-licensed under either:

- **MIT License** - see the [LICENSE-MIT](LICENSE-MIT) file for details
- **Apache License 2.0** - see the [LICENSE-APACHE](LICENSE-APACHE) file for details

You may choose either license at your option.

## Acknowledgments

Built with excellent open-source libraries:

- [Clap](https://crates.io/crates/clap) - Command-line argument parsing with derive macros
- [Rayon](https://crates.io/crates/rayon) - Data parallelism for fast directory scanning
- [Colored](https://crates.io/crates/colored) - Beautiful colored terminal output
- [Indicatif](https://crates.io/crates/indicatif) - Progress bars and spinners
- [Inquire](https://crates.io/crates/inquire) - Interactive prompts and selection
- [WalkDir](https://crates.io/crates/walkdir) - Recursive directory iteration
- [Humansize](https://crates.io/crates/humansize) - Human-readable file sizes

## Support

- **Issues**: [GitHub Issues](https://github.com/TomPlanche/clean-dev-dirs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TomPlanche/clean-dev-dirs/discussions)
- **Crates.io**: [clean-dev-dirs](https://crates.io/crates/clean-dev-dirs)
