# 🧹 nuke-node-modules

A fast, multi-threaded tool to recursively delete `node_modules` directories from your filesystem.

## 🚀 Features

- **🔥 Fast**: Multi-threaded parallel deletion using Rust's performance
- **🛡️ Safe**: Built-in safety checks and confirmation prompts
- **🎯 Flexible**: Exclude patterns support with glob matching
- **🔍 Preview**: Dry-run mode to see what would be deleted
- **📊 Informative**: Detailed progress bars and summary statistics
- **🌈 Beautiful**: Colored output for better UX
- **📦 Cross-platform**: Works on Linux, macOS, and Windows

## 📦 Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone <your-repo-url>
cd nuke-node-modules

# Install globally using Cargo
cargo install --path .
```

After installation, the `nuke-node-modules` command will be available globally in your terminal.

### From crates.io (Future)

```bash
# Once published to crates.io
cargo install nuke-node-modules
```

## 🔧 Usage

### Basic Usage

```bash
# Delete all node_modules in current directory and subdirectories
nuke-node-modules

# Scan a specific directory
nuke-node-modules /path/to/projects

# Preview what would be deleted (dry run)
nuke-node-modules --dry-run

# Skip confirmation prompt
nuke-node-modules --no-confirm

# Quiet mode (minimal output)
nuke-node-modules --quiet
```

### Advanced Options

```bash
# Exclude specific patterns
nuke-node-modules --exclude "**/vendor/**" --exclude "**/build/**"

# Use specific number of threads
nuke-node-modules --threads 8

# Verbose output with directory details
nuke-node-modules --verbose

# Combine options
nuke-node-modules --dry-run --exclude "**/.git/**" --threads 4
```

### Examples

**Preview cleanup:**
```bash
$ nuke-node-modules --dry-run ~/projects
🧹 nuke-node-modules
A fast, multi-threaded node_modules cleanup tool
🔍 DRY RUN MODE - No files will be deleted

📁 Scanning from: /home/user/projects
⚡ Using 8 threads (auto-detected)

Found 15 node_modules directories to delete:
  1. /home/user/projects/frontend
  2. /home/user/projects/backend
  3. /home/user/projects/mobile/ios
  ...and 12 more

🔍 Dry run completed - no files were deleted

📊 Final Summary:
   Found: 15
   Would delete: 15
```

**Real cleanup with exclusions:**
```bash
$ nuke-node-modules --exclude "**/vendor/**" ~/projects
🧹 nuke-node-modules
A fast, multi-threaded node_modules cleanup tool

📁 Scanning from: /home/user/projects
🚫 Exclude patterns:
  - **/vendor/**
⚡ Using 8 threads (auto-detected)

Found 12 node_modules directories:
  1. /home/user/projects/frontend
  2. /home/user/projects/backend
  ...and 10 more

Are you sure you want to delete these directories? [y/N] y
🧹 [████████████████████] 12/12 (0s)
Cleanup complete!

✅ Cleanup completed successfully!

📊 Final Summary:
   Found: 12
   Deleted: 12
   Space freed: 2.3 GB
```

## ⚡ Performance Features

The Rust implementation provides excellent performance characteristics:

- **Multi-threaded deletion** using true parallelism with work-stealing
- **Optimized directory traversal** that stops at node_modules boundaries
- **Memory efficient** streaming with minimal allocation
- **Cross-platform support** for Linux, macOS, and Windows
- **Comprehensive error handling** with detailed reporting
- **Beautiful user interface** with progress bars and colored output

## 🛠️ Development

### Prerequisites

- Rust 1.89.0 or later (see `rust-toolchain.toml`)
- Cargo

### Building

```bash
# Clone the repository
git clone <your-repo-url>
cd nuke-node-modules

# Build in development mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Testing

The project has comprehensive test coverage with:

- **Unit tests** for each module with mocked filesystem operations
- **Integration tests** with real temporary directories
- **Property-based tests** using proptest
- **Performance benchmarks** using criterion

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_exclusion_patterns

# Run benchmarks
cargo bench
```

## 📋 Command Reference

```
USAGE:
    nuke-node-modules [OPTIONS] [PATH]

ARGS:
    <PATH>    Directory to start scanning from (defaults to current directory)

OPTIONS:
    -e, --exclude <PATTERN>  Patterns to exclude from deletion (can be used multiple times)
    -n, --dry-run            Show what would be deleted without actually deleting
    -y, --no-confirm         Skip confirmation prompt
    -q, --quiet              Suppress output (quiet mode)
    -t, --threads <N>        Number of threads to use for parallel deletion
    -v, --verbose            Show detailed information about each directory
    -h, --help               Print help information
    -V, --version            Print version information
```

## 🔒 Safety Features

- **Path validation**: Ensures only `node_modules` directories are deleted
- **Confirmation prompts**: Interactive confirmation before deletion
- **Dry-run mode**: Preview operations without making changes
- **Error handling**: Graceful handling of permission errors
- **Exclusion patterns**: Flexible pattern matching to avoid important directories

---

**⚡ Made with Rust for maximum performance and safety**