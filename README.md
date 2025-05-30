# clean-dev-dirs

A fast and efficient CLI tool for recursively cleaning Rust `target/` and Node.js `node_modules/` directories to reclaim disk space.

## 🚀 Features

- **Multi-language support**: Clean both Rust (`target/`) and Node.js (`node_modules/`) build artifacts
- **Parallel scanning**: Fast directory traversal using multithreading
- **Smart filtering**: Filter by project size, modification time, and project type
- **Interactive mode**: Choose which projects to clean with an interactive interface
- **Dry-run mode**: Preview what would be cleaned without actually deleting anything
- **Progress indicators**: Real-time feedback during scanning and cleaning operations
- **Detailed statistics**: See total space that can be reclaimed before cleaning

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

| Option | Short | Description |
|--------|-------|-------------|
| `--keep-size` | `-s` | Ignore projects with build dir smaller than specified size |
| `--keep-days` | `-d` | Ignore projects modified in the last N days |
| `--rust-only` | | Clean only Rust projects |
| `--node-only` | | Clean only Node.js projects |
| `--yes` | `-y` | Don't ask for confirmation; clean all detected projects |
| `--dry-run` | | List cleanable projects without actually cleaning |
| `--interactive` | `-i` | Use interactive project selection |
| `--threads` | `-t` | Number of threads for directory scanning |
| `--verbose` | `-v` | Show access errors during scanning |
| `--skip` | | Directories to skip during scanning |

## 🎯 Size Formats

The tool supports various size formats:

- **Decimal**: `100KB`, `1.5MB`, `2GB`
- **Binary**: `100KiB`, `1.5MiB`, `2GiB`
- **Bytes**: `1000000`

## 🔍 Project Detection

The tool automatically detects development projects by looking for:

- **Rust projects**: Directories containing both `Cargo.toml` and `target/`
- **Node.js projects**: Directories containing both `package.json` and `node_modules/`

## 🛡️ Safety Features

- **Dry-run mode**: Preview operations before execution
- **Interactive confirmation**: Choose exactly what to clean
- **Intelligent filtering**: Skip recently modified or small projects
- **Error handling**: Graceful handling of permission errors and inaccessible files

## 🎨 Output

The tool provides colored, human-readable output including:

- 🦀 Rust project indicators
- 📦 Node.js project indicators
- 📊 Size statistics in human-readable format
- ✨ Status messages and progress indicators
- 🧪 Dry-run previews

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Clap](https://crates.io/crates/clap) for CLI argument parsing
- Uses [Rayon](https://crates.io/crates/rayon) for parallel processing
- Colored output with [Colored](https://crates.io/crates/colored)
- Progress indicators with [Indicatif](https://crates.io/crates/indicatif) 