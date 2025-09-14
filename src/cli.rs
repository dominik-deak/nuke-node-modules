//! Command-line interface and user interaction

use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::path::{Path, PathBuf};

/// A fast, multi-threaded tool to recursively delete node_modules directories
#[derive(Parser, Debug)]
#[command(name = "nuke-node-modules")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Directory to start scanning from (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Patterns to exclude from deletion (can be used multiple times)
    #[arg(short, long = "exclude", value_name = "PATTERN")]
    pub exclude_patterns: Vec<String>,

    /// Show what would be deleted without actually deleting
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub no_confirm: bool,

    /// Suppress output (quiet mode)
    #[arg(short, long)]
    pub quiet: bool,

    /// Number of threads to use for parallel deletion
    #[arg(short, long, value_name = "N")]
    pub threads: Option<usize>,

    /// Show detailed information about each directory
    #[arg(short, long)]
    pub verbose: bool,
}

impl Cli {
    /// Convert CLI args to Config
    pub fn to_config(&self) -> crate::Config {
        crate::Config {
            exclude_patterns: self.exclude_patterns.clone(),
            dry_run: self.dry_run,
            no_confirm: self.no_confirm,
            quiet: self.quiet,
            threads: self.threads,
        }
    }

    /// Get the root path for scanning
    pub fn get_root_path(&self) -> PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Print the banner with tool information
    pub fn print_banner(&self) {
        if self.quiet {
            return;
        }

        println!("{}", "🧹 nuke-node-modules".bright_cyan().bold());
        println!(
            "{}",
            "A fast, multi-threaded node_modules cleanup tool".dimmed()
        );

        if self.dry_run {
            println!("{}", "🔍 DRY RUN MODE - No files will be deleted".yellow());
        }

        println!();
    }

    /// Print scanning information
    pub fn print_scan_info(&self, root_path: &Path) {
        if self.quiet {
            return;
        }

        println!("📁 Scanning from: {}", root_path.display().to_string().cyan());

        if !self.exclude_patterns.is_empty() {
            println!("🚫 Exclude patterns:");
            for pattern in &self.exclude_patterns {
                println!("  - {}", pattern.yellow());
            }
        }

        if let Some(threads) = self.threads {
            println!("⚡ Using {} threads", threads.to_string().green());
        } else {
            println!(
                "⚡ Using {} threads (auto-detected)",
                num_cpus::get().to_string().green()
            );
        }

        println!();
    }
}

/// Ask user for confirmation before deletion
pub fn confirm_deletion(targets: &[PathBuf]) -> Result<bool> {
    let theme = ColorfulTheme::default();

    println!(
        "{}",
        format!("Found {} node_modules directories:", targets.len())
            .bright_white()
            .bold()
    );

    // Show first few directories, then summarize if there are many
    const MAX_DISPLAY: usize = 10;

    for (i, target) in targets.iter().take(MAX_DISPLAY).enumerate() {
        let parent = target.parent().unwrap_or(target);
        println!("  {}. {}", i + 1, parent.display().to_string().bright_blue());
    }

    if targets.len() > MAX_DISPLAY {
        println!(
            "  ... and {} more",
            (targets.len() - MAX_DISPLAY).to_string().yellow()
        );
    }

    println!();

    let confirmation = Confirm::with_theme(&theme)
        .with_prompt("Are you sure you want to delete these directories?")
        .default(false)
        .interact()?;

    Ok(confirmation)
}

/// Print verbose information about directories
pub fn print_verbose_info(targets: &[PathBuf]) -> Result<()> {
    for (i, target) in targets.iter().enumerate() {
        let parent = target.parent().unwrap_or(target);

        println!("{}. {}", i + 1, parent.display());

        // Try to get some metadata about the directory
        if let Ok(metadata) = std::fs::metadata(target)
            && let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
            let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0);
            if let Some(dt) = datetime {
                println!("   Last modified: {}", dt.format("%Y-%m-%d %H:%M:%S"));
            }
        }

        // Try to estimate size (basic estimation)
        if let Ok(entries) = std::fs::read_dir(target) {
            let count = entries.count();
            if count > 0 {
                println!("   Contains ~{} items", count);
            }
        }

        println!();
    }

    Ok(())
}