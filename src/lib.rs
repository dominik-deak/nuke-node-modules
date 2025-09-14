//! A fast, multi-threaded tool to recursively delete node_modules directories

pub mod scanner;
pub mod cleaner;
pub mod cli;

use anyhow::Result;

/// Configuration for the cleanup operation
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Patterns to exclude from deletion
    pub exclude_patterns: Vec<String>,
    /// Whether to run in dry-run mode (no actual deletion)
    pub dry_run: bool,
    /// Skip confirmation prompt
    pub no_confirm: bool,
    /// Suppress output
    pub quiet: bool,
    /// Number of threads to use (None = auto-detect)
    pub threads: Option<usize>,
}


/// Statistics about the cleanup operation
#[derive(Debug, Default)]
pub struct CleanupStats {
    /// Number of directories found
    pub directories_found: usize,
    /// Number of directories successfully deleted
    pub directories_deleted: usize,
    /// Number of directories skipped due to errors
    pub directories_failed: usize,
    /// Total size freed (in bytes)
    pub bytes_freed: u64,
}

/// Main entry point for the cleanup operation
pub fn cleanup_node_modules<P: AsRef<std::path::Path>>(
    root_path: P,
    config: &Config,
) -> Result<CleanupStats> {
    let scanner = scanner::Scanner::new(root_path, &config.exclude_patterns);
    let targets = scanner.find_node_modules_dirs()?;

    if targets.is_empty() {
        if !config.quiet {
            println!("No node_modules directories found.");
        }
        return Ok(CleanupStats::default());
    }

    if !config.quiet {
        println!("Found {} node_modules directories", targets.len());
        if config.dry_run {
            println!("DRY RUN - would delete:");
        }
        for target in &targets {
            println!("  {}", target.parent().unwrap_or(target).display());
        }
        println!();
    }

    if config.dry_run {
        return Ok(CleanupStats {
            directories_found: targets.len(),
            ..Default::default()
        });
    }

    if !config.no_confirm && !config.quiet
        && !cli::confirm_deletion(&targets)? {
        if !config.quiet {
            println!("Aborted");
        }
        return Ok(CleanupStats {
            directories_found: targets.len(),
            ..Default::default()
        });
    }

    let cleaner = cleaner::Cleaner::new(config.threads, !config.quiet);
    let stats = cleaner.delete_directories(targets)?;

    Ok(stats)
}