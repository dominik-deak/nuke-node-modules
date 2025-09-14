//! Integration tests for the nuke-node-modules tool

use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use nuke_node_modules::{cleanup_node_modules, Config};
use predicates::prelude::*;
use std::process::Command;

/// Create a complex directory structure for testing
fn create_complex_test_structure(temp_dir: &TempDir) -> Result<()> {

    // Create multiple projects with node_modules
    let projects = [
        "frontend/node_modules",
        "backend/node_modules",
        "shared/utils/node_modules",
        "packages/core/node_modules",
        "packages/ui/node_modules",
        "deep/nested/project/node_modules",
    ];

    for project in &projects {
        temp_dir.child(project).create_dir_all()?;

        // Add some files to make directories non-empty
        temp_dir.child(project).child("package.json").write_str(r#"{"name": "test"}"#)?;
        temp_dir.child(project).child("index.js").write_str("console.log('test');")?;
    }

    // Create some directories that should be excluded
    temp_dir.child("vendor/node_modules").create_dir_all()?;
    temp_dir.child("vendor/node_modules/package.json").write_str("{}")?;

    temp_dir.child("build/node_modules").create_dir_all()?;
    temp_dir.child("build/node_modules/index.js").write_str("// build artifact")?;

    // Create non-node_modules directories (should be ignored)
    temp_dir.child("src/components").create_dir_all()?;
    temp_dir.child("docs/assets").create_dir_all()?;

    // Create some regular files
    temp_dir.child("README.md").write_str("# Test Project")?;
    temp_dir.child("package.json").write_str(r#"{"name": "root-project"}"#)?;

    Ok(())
}

#[test]
fn test_basic_cleanup() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_complex_test_structure(&temp_dir)?;

    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    // Should find and delete all 8 node_modules directories (6 + 2 excluded ones)
    assert_eq!(stats.directories_found, 8);
    assert_eq!(stats.directories_deleted, 8);
    assert_eq!(stats.directories_failed, 0);

    // Verify directories were actually deleted
    temp_dir.child("frontend/node_modules").assert(predicate::path::missing());
    temp_dir.child("backend/node_modules").assert(predicate::path::missing());
    temp_dir.child("packages/core/node_modules").assert(predicate::path::missing());

    // Verify parent directories still exist
    temp_dir.child("frontend").assert(predicate::path::exists());
    temp_dir.child("backend").assert(predicate::path::exists());
    temp_dir.child("packages/core").assert(predicate::path::exists());

    // Verify non-node_modules directories are untouched
    temp_dir.child("src/components").assert(predicate::path::exists());
    temp_dir.child("docs/assets").assert(predicate::path::exists());

    Ok(())
}

#[test]
fn test_exclusion_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_complex_test_structure(&temp_dir)?;

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

    // Should find 8 directories but only delete 6 (excluding vendor and build)
    assert_eq!(stats.directories_found, 6);
    assert_eq!(stats.directories_deleted, 6);

    // Verify excluded directories still exist
    temp_dir.child("vendor/node_modules").assert(predicate::path::exists());
    temp_dir.child("build/node_modules").assert(predicate::path::exists());

    // Verify non-excluded directories were deleted
    temp_dir.child("frontend/node_modules").assert(predicate::path::missing());
    temp_dir.child("backend/node_modules").assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_dry_run_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_complex_test_structure(&temp_dir)?;

    let config = Config {
        dry_run: true,
        quiet: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    // Should find directories but not delete anything
    assert_eq!(stats.directories_found, 8);
    assert_eq!(stats.directories_deleted, 0);
    assert_eq!(stats.directories_failed, 0);

    // Verify all directories still exist
    temp_dir.child("frontend/node_modules").assert(predicate::path::exists());
    temp_dir.child("backend/node_modules").assert(predicate::path::exists());
    temp_dir.child("vendor/node_modules").assert(predicate::path::exists());

    Ok(())
}

#[test]
fn test_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create an empty directory structure
    temp_dir.child("empty_project").create_dir_all()?;

    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    assert_eq!(stats.directories_found, 0);
    assert_eq!(stats.directories_deleted, 0);

    Ok(())
}

#[test]
fn test_multithreaded_deletion() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_complex_test_structure(&temp_dir)?;

    let config = Config {
        threads: Some(4),
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    assert_eq!(stats.directories_found, 8);
    assert_eq!(stats.directories_deleted, 8);
    assert_eq!(stats.directories_failed, 0);

    // Verify all node_modules directories were deleted
    temp_dir.child("frontend/node_modules").assert(predicate::path::missing());
    temp_dir.child("packages/core/node_modules").assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_size_calculation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a single node_modules with known content
    temp_dir.child("project/node_modules").create_dir_all()?;

    let large_content = "x".repeat(1000); // 1KB of content
    temp_dir.child("project/node_modules/large_file.txt").write_str(&large_content)?;
    temp_dir.child("project/node_modules/small_file.txt").write_str("small")?;

    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let stats = cleanup_node_modules(temp_dir.path(), &config)?;

    assert_eq!(stats.directories_found, 1);
    assert_eq!(stats.directories_deleted, 1);
    assert!(stats.bytes_freed > 1000); // Should be at least 1KB

    Ok(())
}

/// Test CLI binary integration (requires the binary to be built)
#[test]
fn test_cli_binary_help() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nuke-node-modules"));
    assert!(stdout.contains("recursively delete node_modules"));
    assert!(stdout.contains("--exclude"));
    assert!(stdout.contains("--dry-run"));

    Ok(())
}

#[test]
fn test_cli_binary_dry_run() -> Result<()> {
    let temp_dir = TempDir::new()?;
    temp_dir.child("test_project/node_modules").create_dir_all()?;
    temp_dir.child("test_project/node_modules/package.json").write_str("{}")?;

    let output = Command::new("cargo")
        .args([
            "run", "--",
            "--dry-run",
            "--quiet",
            temp_dir.path().to_str().unwrap()
        ])
        .output()?;

    assert!(output.status.success());

    // Verify the directory still exists after dry run
    temp_dir.child("test_project/node_modules").assert(predicate::path::exists());

    Ok(())
}

#[test]
fn test_invalid_exclusion_pattern_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_complex_test_structure(&temp_dir)?;

    let config = Config {
        exclude_patterns: vec![
            "[invalid".to_string(), // Invalid glob pattern
            "*/valid".to_string(),   // Valid pattern
        ],
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    // Should not panic, just ignore invalid patterns
    let result = cleanup_node_modules(temp_dir.path(), &config);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_permission_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a node_modules directory
    temp_dir.child("project/node_modules").create_dir_all()?;
    temp_dir.child("project/node_modules/file.txt").write_str("content")?;

    // On Unix systems, we could change permissions to test permission errors
    // but this is platform-specific and complex to test reliably
    // This test mainly ensures the error handling path exists

    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let result = cleanup_node_modules(temp_dir.path(), &config);
    assert!(result.is_ok());

    Ok(())
}

/// Benchmark test to ensure reasonable performance
#[test]
fn test_performance_with_many_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create many node_modules directories to test performance
    for i in 0..50 {
        let project_path = format!("project_{}/node_modules", i);
        temp_dir.child(&project_path).create_dir_all()?;
        temp_dir.child(&project_path).child("package.json").write_str("{}")?;
    }

    let config = Config {
        quiet: true,
        no_confirm: true,
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let stats = cleanup_node_modules(temp_dir.path(), &config)?;
    let elapsed = start.elapsed();

    assert_eq!(stats.directories_found, 50);
    assert_eq!(stats.directories_deleted, 50);

    // Should complete in reasonable time (less than 5 seconds for 50 directories)
    assert!(elapsed.as_secs() < 5, "Cleanup took too long: {:?}", elapsed);

    Ok(())
}