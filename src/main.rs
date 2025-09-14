//! Main entry point for the nuke-node-modules CLI tool

use anyhow::Result;
use clap::Parser;
use nuke_node_modules::{cleanup_node_modules, cli::Cli};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Print banner and scanning info
    cli.print_banner();

    let root_path = cli.get_root_path();
    cli.print_scan_info(&root_path);

    // Convert CLI args to config
    let config = cli.to_config();

    // Perform the cleanup
    let stats = cleanup_node_modules(&root_path, &config)?;

    // Print final statistics if not in quiet mode
    if !config.quiet {
        if config.dry_run {
            println!("🔍 Dry run completed - no files were deleted");
        } else if stats.directories_deleted > 0 {
            println!("✅ Cleanup completed successfully!");
        } else if stats.directories_found == 0 {
            println!("ℹ️  No node_modules directories found");
        } else {
            println!("⚠️  Cleanup completed with some issues");
        }

        // Show final summary
        if stats.directories_found > 0 {
            println!();
            println!("📊 Final Summary:");
            println!("   Found: {}", stats.directories_found);

            if config.dry_run {
                println!("   Would delete: {}", stats.directories_found);
            } else {
                println!("   Deleted: {}", stats.directories_deleted);
                if stats.directories_failed > 0 {
                    println!("   Failed: {}", stats.directories_failed);
                }
                if stats.bytes_freed > 0 {
                    println!("   Space freed: {}", format_bytes(stats.bytes_freed));
                }
            }
        }
    }

    // Exit with appropriate code
    if stats.directories_failed > 0 {
        process::exit(1);
    }

    Ok(())
}

/// Format bytes into human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}