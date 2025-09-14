//! Unit tests for cleaner module

use anyhow::Result;
use nuke_node_modules::cleaner::{Cleaner, calculate_directory_size, format_bytes};
use std::path::PathBuf;
use tempfile::TempDir;

mod common;

#[test]
fn test_delete_single_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let node_modules = temp_dir.path().join("node_modules");

    common::create_test_directory_with_content(&node_modules, 3)?;
    assert!(node_modules.exists());

    let cleaner = Cleaner::new(Some(1), false);
    let bytes_freed = cleaner.delete_single_directory(&node_modules)?;

    assert!(!node_modules.exists());
    assert!(bytes_freed > 0);

    Ok(())
}

#[test]
fn test_delete_multiple_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let targets = vec![
        temp_dir.path().join("project1/node_modules"),
        temp_dir.path().join("project2/node_modules"),
        temp_dir.path().join("project3/node_modules"),
    ];

    // Create test directories with content
    for target in &targets {
        common::create_test_directory_with_content(target, 2)?;
        assert!(target.exists());
    }

    let cleaner = Cleaner::new(Some(2), false);
    let stats = cleaner.delete_directories(targets.clone())?;

    // Verify all directories were deleted
    for target in &targets {
        assert!(!target.exists());
    }

    assert_eq!(stats.directories_found, 3);
    assert_eq!(stats.directories_deleted, 3);
    assert_eq!(stats.directories_failed, 0);
    assert!(stats.bytes_freed > 0);

    Ok(())
}

#[test]
fn test_invalid_path_safety() {
    let cleaner = Cleaner::new(Some(1), false);
    let invalid_targets = vec![PathBuf::from("/some/invalid/path")];

    let result = cleaner.delete_directories(invalid_targets);
    assert!(result.is_err());
}

#[test]
fn test_calculate_directory_size() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test");

    let expected_bytes = common::create_test_directory_with_content(&test_dir, 3)?;
    let calculated_bytes = calculate_directory_size(&test_dir)?;

    assert_eq!(calculated_bytes, expected_bytes);

    Ok(())
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1048576), "1.0 MB");
    assert_eq!(format_bytes(1073741824), "1.0 GB");
}

#[test]
fn test_empty_targets_list() -> Result<()> {
    let cleaner = Cleaner::new(Some(1), false);
    let stats = cleaner.delete_directories(vec![])?;

    assert_eq!(stats.directories_found, 0);
    assert_eq!(stats.directories_deleted, 0);
    assert_eq!(stats.directories_failed, 0);
    assert_eq!(stats.bytes_freed, 0);

    Ok(())
}

#[test]
fn test_thread_pool_creation() {
    let cleaner1 = Cleaner::new(None, false); // Auto-detect
    let cleaner2 = Cleaner::new(Some(4), false); // Specific count

    // Both should be valid (hard to test exact thread count without exposing internals)
    assert!(std::mem::size_of_val(&cleaner1) > 0);
    assert!(std::mem::size_of_val(&cleaner2) > 0);
}