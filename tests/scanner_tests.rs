//! Unit tests for scanner module

use anyhow::Result;
use nuke_node_modules::scanner::{Scanner, validate_targets};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

mod common;

#[test]
fn test_find_all_node_modules() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_scanner_test_structure(&temp_dir)?;

    let scanner = Scanner::new(temp_dir.path(), &[]);
    let targets = scanner.find_node_modules_dirs()?;

    assert_eq!(targets.len(), 6); // All node_modules directories

    // Verify all targets end with node_modules
    for target in &targets {
        assert_eq!(target.file_name(), Some("node_modules".as_ref()));
    }

    Ok(())
}

#[test]
fn test_exclusion_patterns() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_scanner_test_structure(&temp_dir)?;

    let exclude_patterns = vec![
        "**/exclude-me/**".to_string(),
        "**/vendor/**".to_string(),
    ];

    let scanner = Scanner::new(temp_dir.path(), &exclude_patterns);
    let targets = scanner.find_node_modules_dirs()?;

    assert_eq!(targets.len(), 4); // Excluded 2 directories

    // Verify excluded paths are not present
    let target_strings: Vec<String> = targets
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    assert!(!target_strings.iter().any(|s| s.contains("exclude-me")));
    assert!(!target_strings.iter().any(|s| s.contains("vendor")));

    Ok(())
}

#[test]
fn test_invalid_exclusion_pattern() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid glob pattern
    let exclude_patterns = vec!["[invalid".to_string()];

    // Should not panic, just skip the invalid pattern
    let scanner = Scanner::new(temp_dir.path(), &exclude_patterns);
    assert_eq!(scanner.exclusion_pattern_count(), 0);
}

#[test]
fn test_validate_targets() -> Result<()> {
    let valid_paths = vec![
        PathBuf::from("/some/path/node_modules"),
        PathBuf::from("/another/node_modules"),
    ];

    assert!(validate_targets(&valid_paths).is_ok());

    let invalid_paths = vec![
        PathBuf::from("/some/path/node_modules"),
        PathBuf::from("/malicious/path"), // This should fail validation
    ];

    assert!(validate_targets(&invalid_paths).is_err());

    Ok(())
}

#[test]
fn test_get_parent_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let scanner = Scanner::new(temp_dir.path(), &[]);

    let paths = vec![
        temp_dir.path().join("project1/node_modules"),
        temp_dir.path().join("project2/node_modules"),
    ];

    let parents = scanner.get_parent_directories(&paths);

    assert_eq!(parents.len(), 2);
    assert_eq!(parents[0].file_name(), Some("project1".as_ref()));
    assert_eq!(parents[1].file_name(), Some("project2".as_ref()));

    Ok(())
}

#[test]
fn test_does_not_traverse_into_node_modules() -> Result<()> {
    let temp_dir = TempDir::new()?;
    common::create_nested_node_modules_structure(&temp_dir)?;

    let scanner = Scanner::new(temp_dir.path(), &[]);
    let targets = scanner.find_node_modules_dirs()?;

    // Should only find the top-level node_modules directories, not nested ones
    assert_eq!(targets.len(), 2, "Should only find 2 top-level node_modules directories");

    let target_strings: Vec<String> = targets
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    // Should contain top-level node_modules
    assert!(target_strings.iter().any(|s| s.ends_with("project/node_modules")));
    assert!(target_strings.iter().any(|s| s.ends_with("other-project/node_modules")));

    // Should NOT contain nested node_modules within packages
    assert!(!target_strings.iter().any(|s| s.contains("some-package/node_modules")));
    assert!(!target_strings.iter().any(|s| s.contains("another-package/node_modules")));
    assert!(!target_strings.iter().any(|s| s.contains("deep-package/sub-package/node_modules")));

    Ok(())
}

#[test]
fn test_only_finds_top_level_node_modules() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a complex structure
    fs::create_dir_all(temp_dir.path().join("app1/node_modules"))?;
    fs::create_dir_all(temp_dir.path().join("app2/node_modules"))?;

    // Create deeply nested node_modules within packages
    fs::create_dir_all(temp_dir.path().join("app1/node_modules/@scope/package/node_modules"))?;
    fs::create_dir_all(temp_dir.path().join("app1/node_modules/react/node_modules"))?;
    fs::create_dir_all(temp_dir.path().join("app2/node_modules/lodash/node_modules"))?;

    // Create a legitimate separate project
    fs::create_dir_all(temp_dir.path().join("workspace/subproject/node_modules"))?;

    let scanner = Scanner::new(temp_dir.path(), &[]);
    let targets = scanner.find_node_modules_dirs()?;

    // Should find exactly 3 top-level node_modules
    assert_eq!(targets.len(), 3, "Should find exactly 3 top-level node_modules directories");

    // Verify they are the correct ones
    let paths: Vec<&str> = targets
        .iter()
        .filter_map(|p| p.to_str())
        .collect();

    let has_app1 = paths.iter().any(|p| p.ends_with("app1/node_modules"));
    let has_app2 = paths.iter().any(|p| p.ends_with("app2/node_modules"));
    let has_subproject = paths.iter().any(|p| p.ends_with("workspace/subproject/node_modules"));

    assert!(has_app1, "Should find app1/node_modules");
    assert!(has_app2, "Should find app2/node_modules");
    assert!(has_subproject, "Should find workspace/subproject/node_modules");

    // Verify we don't find nested package node_modules
    let has_nested = paths.iter().any(|p| {
        p.contains("@scope/package/node_modules")
        || p.contains("react/node_modules")
        || p.contains("lodash/node_modules")
    });
    assert!(!has_nested, "Should not find nested node_modules within packages");

    Ok(())
}

/// Test should_exclude function with various patterns and paths
#[test]
fn test_should_exclude_patterns() -> Result<()> {
    use std::path::Path;

    // Test with exclusion patterns
    let exclude_patterns = vec![
        "**/vendor/**".to_string(),
        "**/build/**".to_string(),
        "**/dist/**".to_string(),
    ];

    let scanner = Scanner::new(".", &exclude_patterns);

    // Should exclude paths matching patterns
    assert!(scanner.should_exclude(Path::new("/project/vendor/lib")));
    assert!(scanner.should_exclude(Path::new("/project/vendor/node_modules")));
    assert!(scanner.should_exclude(Path::new("/app/build/output")));
    assert!(scanner.should_exclude(Path::new("/app/dist/js")));

    // Should not exclude paths that don't match
    assert!(!scanner.should_exclude(Path::new("/project/src")));
    assert!(!scanner.should_exclude(Path::new("/project/node_modules")));
    assert!(!scanner.should_exclude(Path::new("/app/source")));

    Ok(())
}

/// Test should_exclude with no exclusion patterns
#[test]
fn test_should_exclude_no_patterns() {
    use std::path::Path;

    let scanner = Scanner::new(".", &[]);

    // Should not exclude anything when no patterns are set
    assert!(!scanner.should_exclude(Path::new("/project/vendor/lib")));
    assert!(!scanner.should_exclude(Path::new("/project/build/output")));
    assert!(!scanner.should_exclude(Path::new("/any/path/here")));
}

/// Test should_exclude with edge cases
#[test]
fn test_should_exclude_edge_cases() {
    use std::path::Path;

    let exclude_patterns = vec![
        "**/.git/**".to_string(),
        "**/temp_*/**".to_string(),
        "**/*backup*/**".to_string(),
    ];

    let scanner = Scanner::new(".", &exclude_patterns);

    // Test hidden directories
    assert!(scanner.should_exclude(Path::new("/project/.git/hooks")));
    assert!(scanner.should_exclude(Path::new("/app/.git/config")));

    // Test wildcard patterns
    assert!(scanner.should_exclude(Path::new("/project/temp_files/cache")));
    assert!(scanner.should_exclude(Path::new("/project/temp_cache/data")));
    assert!(scanner.should_exclude(Path::new("/project/my_backup_folder/data")));
    assert!(scanner.should_exclude(Path::new("/project/backup_old/files")));

    // Should not exclude similar but not matching paths
    assert!(!scanner.should_exclude(Path::new("/project/template/files")));  // template != temp_*
    assert!(!scanner.should_exclude(Path::new("/project/backup")));  // backup != *backup*
}

/// Test should_exclude with special characters in paths
#[test]
fn test_should_exclude_special_characters() {
    use std::path::Path;

    let exclude_patterns = vec![
        "**/test-*/**".to_string(),
        "**/@scope/**".to_string(),
        "**/node_modules/.cache/**".to_string(),
    ];

    let scanner = Scanner::new(".", &exclude_patterns);

    // Test paths with hyphens
    assert!(scanner.should_exclude(Path::new("/project/test-utils/helper")));
    assert!(scanner.should_exclude(Path::new("/project/test-data/mock")));

    // Test scoped packages
    assert!(scanner.should_exclude(Path::new("/project/@scope/package")));
    assert!(scanner.should_exclude(Path::new("/app/@scope/utils")));

    // Test nested cache paths
    assert!(scanner.should_exclude(Path::new("/project/node_modules/.cache/babel")));
    assert!(scanner.should_exclude(Path::new("/app/node_modules/.cache/webpack")));

    // Should not match non-scoped packages
    assert!(!scanner.should_exclude(Path::new("/project/scope/package")));  // No @ prefix
}