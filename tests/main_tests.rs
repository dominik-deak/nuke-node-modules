//! Tests for main.rs functions

use anyhow::Result;
use nuke_node_modules::{cleanup_node_modules, format_bytes, Config};
use std::fs;
use tempfile::TempDir;

mod common;

/// Test the run function with valid arguments
#[test]
fn test_run_with_valid_path() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    // Simulate CLI args
    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 3);

    Ok(())
}

/// Test format_bytes function edge cases
#[test]
fn test_format_bytes_edge_cases() {
    // Test 0 bytes
    assert_eq!(format_bytes(0), "0 B");

    // Test exactly 1024 boundary
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1023), "1023 B");

    // Test large values
    assert_eq!(format_bytes(1099511627776), "1.0 TB"); // 1 TB

    // Test fractional values
    assert_eq!(format_bytes(1536), "1.5 KB"); // 1.5 KB
    assert_eq!(format_bytes(1572864), "1.5 MB"); // 1.5 MB
}

/// Test with empty directory (no node_modules found)
#[test]
fn test_run_with_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create empty directory structure (no node_modules)
    fs::create_dir_all(temp_dir.path().join("some_project"))?;
    fs::write(temp_dir.path().join("some_project/package.json"), "{}")?;

    let config = Config {
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 0);
    assert_eq!(stats.directories_deleted, 0);

    Ok(())
}

/// Test dry run mode
#[test]
fn test_run_dry_run_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        dry_run: true,
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 0); // Dry run shouldn't delete

    // Verify directories still exist
    assert!(temp_dir.path().join("project1/node_modules").exists());
    assert!(temp_dir.path().join("project2/node_modules").exists());

    Ok(())
}

/// Test verbose output (when not quiet)
#[test]
fn test_run_verbose_output() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        quiet: false, // Enable output
        no_confirm: true,
        ..Default::default()
    };

    // This tests the verbose output paths in main.rs
    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 3);

    Ok(())
}

/// Test exclusion patterns
#[test]
fn test_run_with_exclusions() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create structure with some excluded directories
    fs::create_dir_all(temp_dir.path().join("project1/node_modules"))?;
    fs::create_dir_all(temp_dir.path().join("vendor/node_modules"))?; // Should be excluded
    fs::create_dir_all(temp_dir.path().join("build/node_modules"))?;   // Should be excluded

    let config = Config {
        exclude_patterns: vec![
            "**/vendor/**".to_string(),
            "**/build/**".to_string(),
        ],
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 1); // Only found 1 (excluded 2)
    assert_eq!(stats.directories_deleted, 1);

    // Verify excluded directories still exist
    assert!(temp_dir.path().join("vendor/node_modules").exists());
    assert!(temp_dir.path().join("build/node_modules").exists());

    // Verify non-excluded was deleted
    assert!(!temp_dir.path().join("project1/node_modules").exists());

    Ok(())
}

/// Test statistics output formatting
#[test]
fn test_statistics_formatting() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a node_modules with known size
    let node_modules = temp_dir.path().join("project/node_modules");
    fs::create_dir_all(&node_modules)?;
    fs::write(node_modules.join("package.json"), "{}")?;
    fs::write(node_modules.join("large_file.txt"), "x".repeat(2048))?; // 2KB

    let config = Config {
        quiet: false, // Enable statistics output
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 1);
    assert_eq!(stats.directories_deleted, 1);
    assert!(stats.bytes_freed > 2000); // At least 2KB

    Ok(())
}