//! Unit tests for lib module and main functions

use anyhow::Result;
use nuke_node_modules::{cleanup_node_modules, Config};
use std::fs;
use tempfile::TempDir;

mod common;

#[test]
fn test_dry_run_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        dry_run: true,
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 0);

    // Verify nothing was actually deleted
    assert!(temp_dir.path().join("project1/node_modules").exists());
    assert!(temp_dir.path().join("project2/node_modules").exists());

    Ok(())
}

#[test]
fn test_no_node_modules_found() -> Result<()> {
    let temp_dir = TempDir::new()?;
    fs::create_dir_all(temp_dir.path().join("empty_project"))?;

    let config = Config {
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    assert_eq!(stats.directories_found, 0);
    assert_eq!(stats.directories_deleted, 0);

    Ok(())
}

// Tests for main.rs functions
#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1048576), "1.0 MB");
    assert_eq!(format_bytes(1073741824), "1.0 GB");
}

/// Format bytes into human-readable format (copied from main.rs for testing)
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

// Note: Testing the main function would require more complex integration testing
// since it involves CLI parsing, file system operations, and process exit codes