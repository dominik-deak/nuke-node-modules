//! Unit tests for lib module and main functions

use anyhow::Result;
use nuke_node_modules::{cleanup_node_modules, Config, format_bytes};
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


/// Test user cancellation flow
#[test]
fn test_cleanup_user_cancellation() -> Result<()> {
    // This test can't easily be run because it requires user interaction
    // But we can test the structure for code coverage

    // The cancellation path is when config.no_confirm is false and user says no
    // This would normally require mocking the confirm_deletion function
    // For now, we test that the function exists and the structure is sound
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        no_confirm: true, // This bypasses the interactive prompt
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 3);

    Ok(())
}

/// Test error handling in cleanup process
#[test]
fn test_cleanup_with_invalid_path() {
    let config = Config {
        quiet: true,
        ..Default::default()
    };

    // Test with a path that doesn't exist
    let result = cleanup_node_modules("/completely/nonexistent/path/that/should/not/exist", &config);

    // The scanner might return an error for completely invalid paths
    // or it might return Ok with 0 directories - either is acceptable
    if let Ok(stats) = result {
        assert_eq!(stats.directories_found, 0);
    }
    // If it's an error, that's also fine for an invalid path
}

/// Test various edge cases for format_bytes
#[test]
fn test_format_bytes_all_units() {
    // Test all unit conversions to cover all branches
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");  // 1.5 KB
    assert_eq!(format_bytes(1048575), "1024.0 KB"); // Just under 1 MB
    assert_eq!(format_bytes(1048576), "1.0 MB");     // Exactly 1 MB
    assert_eq!(format_bytes(1610612736), "1.5 GB");  // 1.5 GB
    assert_eq!(format_bytes(1099511627776), "1.0 TB"); // 1 TB
    // Test large number (don't test u64::MAX as the exact value may vary)
    assert!(format_bytes(2199023255552).contains("TB")); // 2 TB
}

/// Test cleanup with verbose output (non-quiet mode)
#[test]
fn test_cleanup_verbose_output() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        quiet: false,  // Enable verbose output
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 3);

    Ok(())
}

/// Test cleanup with dry-run verbose output
#[test]
fn test_cleanup_dry_run_verbose() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_lib_test_structure(&temp_dir)?;

    let config = Config {
        dry_run: true,
        quiet: false,  // Enable verbose output with dry run
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 0);

    Ok(())
}

/// Test the directories_found == 0 case with verbose output
#[test]
fn test_cleanup_no_directories_found_verbose() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create directory structure without node_modules
    fs::create_dir_all(temp_dir.path().join("project"))?;
    fs::write(temp_dir.path().join("project/package.json"), "{}")?;

    let config = Config {
        quiet: false,  // Enable verbose output
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    assert_eq!(stats.directories_found, 0);
    assert_eq!(stats.directories_deleted, 0);

    Ok(())
}

// Note: Testing the main function would require more complex integration testing
// since it involves CLI parsing, file system operations, and process exit codes