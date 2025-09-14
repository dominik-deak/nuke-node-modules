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