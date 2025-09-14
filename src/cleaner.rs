//! Parallel directory deletion functionality

use crate::{scanner, CleanupStats, format_bytes};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

/// Cleaner for parallel directory deletion
pub struct Cleaner {
    thread_pool: rayon::ThreadPool,
    show_progress: bool,
}


impl Cleaner {
    /// Create a new cleaner with specified thread count
    pub fn new(threads: Option<usize>, show_progress: bool) -> Self {
        let num_threads = threads.unwrap_or_else(num_cpus::get);

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to create thread pool");

        // Disable progress bars when running tests
        let show_progress = show_progress && !Self::is_test_environment();

        Self {
            thread_pool,
            show_progress,
        }
    }

    /// Check if we're running in a test environment
    pub fn is_test_environment() -> bool {
        // Compile-time test detection
        if cfg!(test) {
            return true;
        }

        // Runtime test detection via environment variables
        std::env::var("CARGO_MANIFEST_DIR").is_ok() && (
            std::env::var("RUST_TEST").is_ok() ||
            std::env::var("CARGO_CRATE_NAME").is_ok() ||
            std::env::args().any(|arg| arg.contains("test"))
        )
    }

    /// Delete directories in parallel
    pub fn delete_directories(&self, targets: Vec<PathBuf>) -> Result<CleanupStats> {
        // Safety check - ensure all paths end with node_modules
        scanner::validate_targets(&targets)?;

        if targets.is_empty() {
            return Ok(CleanupStats::default());
        }

        let progress_bar = if self.show_progress {
            let pb = ProgressBar::new(targets.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")?
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };

        // Atomic counters for thread-safe statistics
        let deleted_count = AtomicUsize::new(0);
        let failed_count = AtomicUsize::new(0);
        let bytes_freed = AtomicU64::new(0);
        let errors = Mutex::new(Vec::new());

        // Execute deletions in parallel
        self.thread_pool.install(|| {
            targets
                .par_iter()
                .for_each(|target| {
                    let result = self.delete_single_directory(target);

                    // Update progress bar
                    if let Some(ref pb) = progress_bar {
                        pb.inc(1);
                    }

                    // Update counters
                    match result {
                        Ok(bytes) => {
                            deleted_count.fetch_add(1, Ordering::Relaxed);
                            bytes_freed.fetch_add(bytes, Ordering::Relaxed);
                        }
                        Err(e) => {
                            failed_count.fetch_add(1, Ordering::Relaxed);
                            if let Ok(mut errors) = errors.lock() {
                                errors.push(format!("{}: {}", target.display(), e));
                            }
                        }
                    }
                })
        });

        if let Some(pb) = progress_bar {
            pb.finish_with_message("Cleanup complete!");
        }

        // Print errors if any occurred
        if let Ok(error_list) = errors.lock() {
            if !error_list.is_empty() && self.show_progress {
                eprintln!("\nErrors encountered:");
                for error in error_list.iter() {
                    eprintln!("  {}", error);
                }
            }
        }

        let stats = CleanupStats {
            directories_found: targets.len(),
            directories_deleted: deleted_count.load(Ordering::Relaxed),
            directories_failed: failed_count.load(Ordering::Relaxed),
            bytes_freed: bytes_freed.load(Ordering::Relaxed),
        };

        if self.show_progress {
            print_cleanup_summary(&stats);
        }

        Ok(stats)
    }

    /// Delete a single directory and return bytes freed
    pub fn delete_single_directory(&self, path: &Path) -> Result<u64> {
        // Calculate size before deletion (for statistics)
        let size_before = calculate_directory_size(path).unwrap_or(0);

        // Perform the deletion
        fs::remove_dir_all(path)?;

        Ok(size_before)
    }
}

/// Calculate the total size of a directory and its contents
pub fn calculate_directory_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total_size += entry.metadata()?.len();
        }
    }

    Ok(total_size)
}

/// Print a summary of the cleanup operation
pub fn print_cleanup_summary(stats: &CleanupStats) {
    println!("\n🧹 Cleanup Summary:");
    println!("  Directories found: {}", stats.directories_found);
    println!("  Successfully deleted: {}", stats.directories_deleted);

    if stats.directories_failed > 0 {
        println!("  Failed to delete: {}", stats.directories_failed);
    }

    if stats.bytes_freed > 0 {
        println!("  Space freed: {}", format_bytes(stats.bytes_freed));
    }
}

