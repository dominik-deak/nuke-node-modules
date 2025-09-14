//! Unit tests for CLI module

use clap::Parser;
use nuke_node_modules::cli::Cli;
use std::path::PathBuf;

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

// Note: Interactive tests (confirm_deletion, print functions) would require
// more complex mocking or integration testing setup