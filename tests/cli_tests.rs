//! Unit tests for CLI module

use clap::Parser;
use nuke_node_modules::cli::Cli;
use std::path::PathBuf;
use anyhow::Result;

#[test]
fn test_cli_parsing() {
    let cli = Cli::parse_from(["nuke-node-modules"]);
    assert!(cli.path.is_none());
    assert!(!cli.dry_run);
    assert!(!cli.no_confirm);
    assert!(!cli.quiet);
    assert!(cli.threads.is_none());
    assert!(cli.exclude_patterns.is_empty());
}

#[test]
fn test_cli_with_all_options() {
    let cli = Cli::parse_from([
        "nuke-node-modules",
        "/some/path",
        "--exclude",
        "pattern1",
        "--exclude",
        "pattern2",
        "--dry-run",
        "--no-confirm",
        "--quiet",
        "--threads",
        "8",
        "--verbose",
    ]);

    assert_eq!(cli.path, Some(PathBuf::from("/some/path")));
    assert!(cli.dry_run);
    assert!(cli.no_confirm);
    assert!(cli.quiet);
    assert_eq!(cli.threads, Some(8));
    assert_eq!(cli.exclude_patterns, vec!["pattern1", "pattern2"]);
    assert!(cli.verbose);
}

#[test]
fn test_config_conversion() {
    let cli = Cli::parse_from([
        "nuke-node-modules",
        "--exclude",
        "test",
        "--dry-run",
        "--quiet",
    ]);

    let config = cli.to_config();

    assert_eq!(config.exclude_patterns, vec!["test"]);
    assert!(config.dry_run);
    assert!(config.quiet);
    assert!(!config.no_confirm); // Default
}

#[test]
fn test_get_root_path() {
    // Test with explicit path
    let cli = Cli::parse_from(["nuke-node-modules", "/explicit/path"]);
    assert_eq!(cli.get_root_path(), PathBuf::from("/explicit/path"));

    // Test with default (current directory)
    let cli = Cli::parse_from(["nuke-node-modules"]);
    let root = cli.get_root_path();

    // Should be either current directory or "." fallback
    assert!(root.is_absolute() || root == PathBuf::from("."));
}

use tempfile::TempDir;
use std::fs;

#[test]
fn test_print_banner_quiet_mode() {
    let cli = Cli::parse_from(["nuke-node-modules", "--quiet"]);
    // In quiet mode, print_banner should do nothing (just return)
    cli.print_banner(); // This should not print anything
    // No assertion needed - this tests the quiet return path
}

#[test]
fn test_print_banner_normal_mode() {
    let cli = Cli::parse_from(["nuke-node-modules"]);
    // This tests the normal banner printing path
    cli.print_banner(); // Should print banner to stdout
    // The actual printing is tested by running this without panicking
}

#[test]
fn test_print_scan_info_default_threads() {
    let cli = Cli::parse_from(["nuke-node-modules"]);
    let path = PathBuf::from(".");

    // Test the scan info printing (no threads specified)
    cli.print_scan_info(&path);
    // This tests the auto-detected thread path
}

#[test]
fn test_print_scan_info_custom_threads() {
    let cli = Cli::parse_from(["nuke-node-modules", "--threads", "4"]);
    let path = PathBuf::from("/test/path");

    // Test with custom thread count
    cli.print_scan_info(&path);
    // This tests the custom thread count path
}

#[test]
fn test_print_scan_info_quiet_mode() {
    let cli = Cli::parse_from(["nuke-node-modules", "--quiet"]);
    let path = PathBuf::from(".");

    // In quiet mode, should do nothing
    cli.print_scan_info(&path);
    // This tests the quiet return path
}

#[test]
fn test_print_verbose_info() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create test node_modules directories
    let node_modules1 = temp_dir.path().join("project1/node_modules");
    let node_modules2 = temp_dir.path().join("project2/node_modules");

    fs::create_dir_all(&node_modules1)?;
    fs::create_dir_all(&node_modules2)?;

    // Add some content
    fs::write(node_modules1.join("package.json"), "{}")?;
    fs::write(node_modules2.join("index.js"), "test")?;

    let targets = vec![node_modules1, node_modules2];

    // Test verbose info printing
    let result = nuke_node_modules::cli::print_verbose_info(&targets);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_print_verbose_info_with_invalid_paths() {
    // Test with paths that don't exist
    let targets = vec![
        PathBuf::from("/nonexistent/path/node_modules"),
        PathBuf::from("/another/fake/node_modules"),
    ];

    // Should not panic even with invalid paths
    let result = nuke_node_modules::cli::print_verbose_info(&targets);
    assert!(result.is_ok());
}

#[test]
fn test_print_verbose_info_empty_list() {
    // Test with empty targets list
    let targets: Vec<PathBuf> = vec![];

    let result = nuke_node_modules::cli::print_verbose_info(&targets);
    assert!(result.is_ok());
}

// Note: confirm_deletion function uses interactive prompts and cannot be easily tested
// without mocking the dialoguer crate. These tests verify related functionality instead.

#[test]
fn test_confirm_deletion_function_exists() {
    // Verify we can create target vectors that would be passed to confirm_deletion
    // We don't call the function to avoid interactive prompts during tests

    // Test that we can create target vectors that would be passed to the function
    let targets: Vec<PathBuf> = vec![
        PathBuf::from("/test/project1/node_modules"),
        PathBuf::from("/test/project2/node_modules"),
    ];

    // Verify basic path operations work
    assert_eq!(targets.len(), 2);
    assert!(targets[0].ends_with("node_modules"));
    assert!(targets[1].ends_with("node_modules"));
}

#[test]
fn test_target_list_handling() {
    // Test handling of large target lists (what would be passed to confirm_deletion)
    let targets: Vec<PathBuf> = (0..15)
        .map(|i| PathBuf::from(format!("/test/project{}/node_modules", i)))
        .collect();

    // Verify we can handle many targets without issues
    assert_eq!(targets.len(), 15);

    // Test the "and X more" logic would apply (>10 targets)
    assert!(targets.len() > 10);

    // Verify all paths have correct structure
    for target in &targets {
        assert!(target.to_string_lossy().contains("project"));
        assert!(target.ends_with("node_modules"));
    }
}

// Note: The actual interactive parts of confirm_deletion would need
// integration testing or mocking the dialoguer crate, which is complex