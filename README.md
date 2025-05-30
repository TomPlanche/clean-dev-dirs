# 🧹clean-dev-dirs

> A fast and efficient CLI tool for recursively cleaning Rust `target/`, Node.js `node_modules/`, Python cache, and Go `vendor/` directories to reclaim disk space.

<p align="center">
  <a href="https://crates.io/crates/clean-dev-dirs"><img src="https://img.shields.io/crates/v/clean-dev-dirs.svg" alt="Crates.io Version"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=alert_status" alt="SonarCloud Status"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=sqale_rating" alt="SonarCloud SQALE Rating"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_clean-dev-dirs"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_clean-dev-dirs&metric=security_rating" alt="SonarCloud Security Rating"></a>
  <a href="https://github.com/TomPlanche/clean-dev-dirs/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clean-dev-dirs" alt="License"></a>
</p>

## 🚀 Features

- **Multi-language support**: Clean Rust (`target/`), Node.js (`node_modules/`), Python (cache dirs), and Go (`vendor/`) build artifacts
- **Parallel scanning**: Fast directory traversal using multithreading
- **Smart filtering**: Filter by project size, modification time, and project type
- **Interactive mode**: Choose which projects to clean with an interactive interface
- **Dry-run mode**: Preview what would be cleaned without actually deleting anything
- **Progress indicators**: Real-time feedback during scanning and cleaning operations
- **Detailed statistics**: See total space that can be reclaimed before cleaning

## 💡 Inspiration

This project is inspired by [cargo-clean-all](https://github.com/dnlmlr/cargo-clean-all), a Rust-specific tool for
cleaning cargo projects. I've improved upon the original concept with:

- **Multi-language support**: Extended beyond Rust to support Node.js, Python, and Go projects
- **Parallel scanning**: Significantly faster directory traversal using multithreading
- **Cleaner code architecture**: Well-structured, modular codebase for better maintainability

## 📦 Installation

### From Source

```bash
git clone https://github.com/your-username/clean-dev-dirs.git
cd clean-dev-dirs
cargo install --path .
```

### Using Cargo

```bash
cargo install clean-dev-dirs
```

## 🛠 Usage

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

### Filtering Options

```bash
# Only clean projects larger than 100MB
clean-dev-dirs --keep-size 100MB

# Only clean projects not modified in the last 30 days
clean-dev-dirs --keep-days 30

# Clean only Rust projects
clean-dev-dirs --rust-only

# Clean only Node.js projects
clean-dev-dirs --node-only

# Clean only Python projects
clean-dev-dirs --python-only

# Clean only Go projects
clean-dev-dirs --go-only
```

### Advanced Options

```bash
# Use 8 threads for scanning
clean-dev-dirs --threads 8

# Show verbose output including scan errors
clean-dev-dirs --verbose

# Skip specific directories
clean-dev-dirs --skip node_modules --skip .git

# Non-interactive mode (auto-confirm)
clean-dev-dirs --yes
```
## 📋 Command Line Options

| Option          | Short | Description                                                |
|-----------------|-------|------------------------------------------------------------|
| `--keep-size`   | `-s`  | Ignore projects with build dir smaller than specified size |
| `--keep-days`   | `-d`  | Ignore projects modified in the last N days                |
| `--rust-only`   |       | Clean only Rust projects                                   |
| `--node-only`   |       | Clean only Node.js projects                                |
| `--python-only` |       | Clean only Python projects                                 |
| `--go-only`     |       | Clean only Go projects                                     |
| `--yes`         | `-y`  | Don't ask for confirmation; clean all detected projects    |
| `--dry-run`     |       | List cleanable projects without actually cleaning          |
| `--interactive` | `-i`  | Use interactive project selection                          |
| `--threads`     | `-t`  | Number of threads for directory scanning                   |
| `--verbose`     | `-v`  | Show access errors during scanning                         |
| `--skip`        |       | Directories to skip during scanning                        |

## 🎯 Size Formats

The tool supports various size formats:

- **Decimal**: `100KB`, `1.5MB`, `2GB`
- **Binary**: `100KiB`, `1.5MiB`, `2GiB`
- **Bytes**: `1000000`

## 🔍 Project Detection

The tool automatically detects development projects by looking for:

- **Rust projects**: Directories containing both `Cargo.toml` and `target/`
- **Node.js projects**: Directories containing both `package.json` and `node_modules/`
- **Python projects**: Directories containing Python config files (`requirements.txt`, `setup.py`, `pyproject.toml`) and cache dirs (`__pycache__`, `.pytest_cache`, `venv`, etc.)
- **Go projects**: Directories containing both `go.mod` and `vendor/`

## 🛡️ Safety Features

- **Dry-run mode**: Preview operations before execution
- **Interactive confirmation**: Choose exactly what to clean
- **Intelligent filtering**: Skip recently modified or small projects
- **Error handling**: Graceful handling of permission errors and inaccessible files

## 🎨 Output

The tool provides colored, human-readable output including:

- 🦀 Rust project indicators
- 📦 Node.js project indicators
- 🐍 Python project indicators
- 🐹 Go project indicators
- 📊 Size statistics in human-readable format
- ✨ Status messages and progress indicators
- 🧪 Dry-run previews

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### 🔧 Adding Language Support

Want to add support for a new programming language? Here's how to extend `clean-dev-dirs`:

#### 1. **Update Project Types**

First, add your language to the `ProjectType` enum in `src/project/project.rs`:

```rust
#[derive(Clone, PartialEq)]
pub(crate) enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    YourLanguage, // Add your language here
}
```

Don't forget to update the `Display` implementation to include an appropriate emoji and name.

#### 2. **Add CLI Filter Option**

Update `src/cli.rs` to add a command-line flag for your language:

```rust
struct ProjectTypeArgs {
    // ... existing flags ...
    
    /// Clean only YourLanguage projects
    #[arg(long, conflicts_with_all = ["rust_only", "node_only", "python_only", "go_only"])]
    your_language_only: bool,
}
```

Also update the `ProjectFilter` enum and `project_filter()` method accordingly.

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
- Add an example in the usage section

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

## 📄 License

This project is dual-licensed under either:

- **MIT License** - see the [LICENSE-MIT](LICENSE-MIT) file for details
- **Apache License 2.0** - see the [LICENSE-APACHE](LICENSE-APACHE) file for details

You may choose either license at your option.

## 🙏 Acknowledgments

- Built with [Clap](https://crates.io/crates/clap) for CLI argument parsing
- Uses [Rayon](https://crates.io/crates/rayon) for parallel processing
- Colored output with [Colored](https://crates.io/crates/colored)
- Progress indicators with [Indicatif](https://crates.io/crates/indicatif) 
