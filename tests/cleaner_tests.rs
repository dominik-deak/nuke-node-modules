//! Unit tests for cleaner module

use anyhow::Result;
use nuke_node_modules::cleaner::{Cleaner, calculate_directory_size};
use nuke_node_modules::format_bytes;
use std::path::PathBuf;
use std::fs;
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

/// Test cleaner with progress display enabled
#[test]
fn test_cleaner_with_progress() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let node_modules = temp_dir.path().join("node_modules");
    common::create_test_directory_with_content(&node_modules, 2)?;

    let cleaner = Cleaner::new(Some(1), true); // With progress display
    let stats = cleaner.delete_directories(vec![node_modules])?;

    assert_eq!(stats.directories_found, 1);
    assert_eq!(stats.directories_deleted, 1);

    Ok(())
}

/// Test delete_single_directory method directly
#[test]
fn test_delete_single_directory_method() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test_directory");

    // Create directory with content
    fs::create_dir_all(&test_dir)?;
    fs::write(test_dir.join("file1.txt"), "content1")?;
    fs::write(test_dir.join("file2.txt"), "content2")?;

    assert!(test_dir.exists());

    let cleaner = Cleaner::new(Some(1), false);
    let bytes_freed = cleaner.delete_single_directory(&test_dir)?;
    assert!(!test_dir.exists());
    assert!(bytes_freed > 0);

    Ok(())
}

/// Test delete_single_directory method with non-existent path
#[test]
fn test_delete_single_directory_method_nonexistent() {
    let nonexistent = PathBuf::from("/path/that/does/not/exist");
    let cleaner = Cleaner::new(Some(1), false);
    let result = cleaner.delete_single_directory(&nonexistent);
    assert!(result.is_err());
}

/// Test error handling with mixed valid/invalid paths
#[test]
fn test_delete_mixed_valid_invalid_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create one valid directory
    let valid_dir = temp_dir.path().join("valid/node_modules");
    fs::create_dir_all(&valid_dir)?;
    fs::write(valid_dir.join("package.json"), "{}")?;

    // One invalid directory
    let invalid_dir = PathBuf::from("/nonexistent/node_modules");

    let cleaner = Cleaner::new(Some(1), false);
    let targets = vec![valid_dir, invalid_dir];

    let stats = cleaner.delete_directories(targets)?;

    // Should have tried both, succeeded on one, failed on one
    assert_eq!(stats.directories_found, 2);
    assert_eq!(stats.directories_deleted, 1);
    assert_eq!(stats.directories_failed, 1);

    Ok(())
}

/// Test size calculation error handling
#[test]
fn test_calculate_directory_size_error() {
    let nonexistent = PathBuf::from("/path/that/does/not/exist");
    let result = calculate_directory_size(&nonexistent);
    assert!(result.is_err());
}

/// Test large directory with many files
#[test]
fn test_delete_large_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let node_modules = temp_dir.path().join("large_project/node_modules");

    // Create directory with many small files
    fs::create_dir_all(&node_modules)?;
    for i in 0..50 {
        fs::write(node_modules.join(format!("file_{}.js", i)), format!("content {}", i))?;
    }

    // Create subdirectories
    for i in 0..5 {
        let subdir = node_modules.join(format!("package_{}", i));
        fs::create_dir_all(&subdir)?;
        fs::write(subdir.join("index.js"), format!("module {}", i))?;
    }

    let cleaner = Cleaner::new(Some(2), false);
    let stats = cleaner.delete_directories(vec![node_modules.clone()])?;

    assert_eq!(stats.directories_found, 1);
    assert_eq!(stats.directories_deleted, 1);
    assert!(!node_modules.exists());
    assert!(stats.bytes_freed > 0);

    Ok(())
}

/// Test concurrent deletion with multiple threads
#[test]
fn test_concurrent_deletion() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create multiple directories
    let mut targets = vec![];
    for i in 0..8 {
        let target = temp_dir.path().join(format!("project_{}/node_modules", i));
        common::create_test_directory_with_content(&target, 2)?;
        targets.push(target);
    }

    let cleaner = Cleaner::new(Some(4), false); // Use 4 threads
    let stats = cleaner.delete_directories(targets)?;

    assert_eq!(stats.directories_found, 8);
    assert_eq!(stats.directories_deleted, 8);
    assert_eq!(stats.directories_failed, 0);

    Ok(())
}

/// Test is_test_environment function
#[test]
fn test_is_test_environment() {
    // When running tests, cfg!(test) is true, so this should always return true
    assert!(Cleaner::is_test_environment());
}

/// Test is_test_environment with environment variables
#[test]
fn test_is_test_environment_with_env_vars() {
    use std::env;

    // Save original environment state
    let original_cargo_manifest = env::var("CARGO_MANIFEST_DIR").ok();
    let original_rust_test = env::var("RUST_TEST").ok();
    let original_cargo_crate = env::var("CARGO_CRATE_NAME").ok();

    // Test with CARGO_MANIFEST_DIR set (which it typically is during tests)
    if original_cargo_manifest.is_none() {
        unsafe { env::set_var("CARGO_MANIFEST_DIR", "/test/path"); }
    }

    // The function should still return true due to cfg!(test) being true
    // during test compilation, regardless of environment variables
    assert!(Cleaner::is_test_environment());

    // Restore original environment
    unsafe {
        match original_cargo_manifest {
            Some(value) => env::set_var("CARGO_MANIFEST_DIR", value),
            None => env::remove_var("CARGO_MANIFEST_DIR"),
        }
        match original_rust_test {
            Some(value) => env::set_var("RUST_TEST", value),
            None => env::remove_var("RUST_TEST"),
        }
        match original_cargo_crate {
            Some(value) => env::set_var("CARGO_CRATE_NAME", value),
            None => env::remove_var("CARGO_CRATE_NAME"),
        }
    }
}

/// Test is_test_environment behavior during test compilation
#[test]
fn test_is_test_environment_cfg_test() {
    // This test verifies that cfg!(test) detection works
    // During test compilation, cfg!(test) is always true
    // so is_test_environment() should return true

    // We can't test the "false" case in unit tests since cfg!(test) is always true
    // but we can document the expected behavior
    assert!(Cleaner::is_test_environment(),
        "During test compilation, cfg!(test) is true, so function should return true");
}

/// Test print_cleanup_summary function
#[test]
fn test_print_cleanup_summary() {
    use nuke_node_modules::cleaner::print_cleanup_summary;
    use nuke_node_modules::CleanupStats;

    // Test with basic stats
    let stats = CleanupStats {
        directories_found: 5,
        directories_deleted: 4,
        directories_failed: 1,
        bytes_freed: 1024 * 1024, // 1 MB
    };

    // Function should not panic
    print_cleanup_summary(&stats);
}

/// Test print_cleanup_summary with zero values
#[test]
fn test_print_cleanup_summary_zero_values() {
    use nuke_node_modules::cleaner::print_cleanup_summary;
    use nuke_node_modules::CleanupStats;

    // Test with all zeros
    let stats = CleanupStats {
        directories_found: 0,
        directories_deleted: 0,
        directories_failed: 0,
        bytes_freed: 0,
    };

    // Function should not panic with zero values
    print_cleanup_summary(&stats);
}

/// Test print_cleanup_summary with large values
#[test]
fn test_print_cleanup_summary_large_values() {
    use nuke_node_modules::cleaner::print_cleanup_summary;
    use nuke_node_modules::CleanupStats;

    // Test with large numbers
    let stats = CleanupStats {
        directories_found: 1000,
        directories_deleted: 999,
        directories_failed: 1,
        bytes_freed: 1024 * 1024 * 1024 * 5, // 5 GB
    };

    // Function should not panic with large values
    print_cleanup_summary(&stats);
}

/// Test print_cleanup_summary with no failures
#[test]
fn test_print_cleanup_summary_no_failures() {
    use nuke_node_modules::cleaner::print_cleanup_summary;
    use nuke_node_modules::CleanupStats;

    // Test with no failures (common success case)
    let stats = CleanupStats {
        directories_found: 10,
        directories_deleted: 10,
        directories_failed: 0,
        bytes_freed: 512 * 1024, // 512 KB
    };

    // Function should not panic with no failures
    print_cleanup_summary(&stats);
}

/// Test print_cleanup_summary with no bytes freed
#[test]
fn test_print_cleanup_summary_no_bytes_freed() {
    use nuke_node_modules::cleaner::print_cleanup_summary;
    use nuke_node_modules::CleanupStats;

    // Test with directories deleted but no bytes freed
    let stats = CleanupStats {
        directories_found: 3,
        directories_deleted: 3,
        directories_failed: 0,
        bytes_freed: 0, // Empty directories
    };

    // Function should not panic when no bytes are freed
    print_cleanup_summary(&stats);
}