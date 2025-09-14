//! Shared test utilities for nuke-node-modules

#![allow(dead_code)]

use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a basic test structure with various node_modules directories for scanner tests
pub fn create_scanner_test_structure(temp_dir: &TempDir) -> Result<()> {
    let base = temp_dir.path();

    // Create various node_modules directories
    fs::create_dir_all(base.join("project1/node_modules"))?;
    fs::create_dir_all(base.join("project2/node_modules"))?;
    fs::create_dir_all(base.join("nested/project3/node_modules"))?;
    fs::create_dir_all(base.join("deep/nested/project4/node_modules"))?;

    // Create some directories that should be excluded
    fs::create_dir_all(base.join("exclude-me/node_modules"))?;
    fs::create_dir_all(base.join("vendor/node_modules"))?;

    // Create some non-node_modules directories that should be ignored
    fs::create_dir_all(base.join("regular_dir"))?;
    fs::create_dir_all(base.join("src/components"))?;

    Ok(())
}

/// Create a test structure with nested node_modules to test traversal behavior
pub fn create_nested_node_modules_structure(temp_dir: &TempDir) -> Result<()> {
    let base = temp_dir.path();

    // Create main project with node_modules
    fs::create_dir_all(base.join("project/node_modules"))?;

    // Create nested node_modules within packages (should NOT be found)
    fs::create_dir_all(base.join("project/node_modules/some-package/node_modules"))?;
    fs::create_dir_all(base.join("project/node_modules/some-package/node_modules/nested-dep"))?;
    fs::create_dir_all(base.join("project/node_modules/another-package/node_modules"))?;
    fs::create_dir_all(base.join("project/node_modules/deep-package/sub-package/node_modules"))?;

    // Create another top-level project
    fs::create_dir_all(base.join("other-project/node_modules"))?;
    fs::create_dir_all(base.join("other-project/node_modules/package/node_modules"))?;

    // Add some files to make directories realistic
    fs::write(base.join("project/node_modules/some-package/package.json"), "{}")?;
    fs::write(base.join("project/node_modules/some-package/node_modules/package.json"), "{}")?;

    Ok(())
}

/// Create a test directory with specified number of files and return total bytes written
pub fn create_test_directory_with_content(path: &Path, file_count: usize) -> Result<u64> {
    fs::create_dir_all(path)?;
    let mut total_bytes = 0;

    for i in 0..file_count {
        let file_path = path.join(format!("file_{}.txt", i));
        let content = format!("content for file {}", i);
        fs::write(&file_path, &content)?;
        total_bytes += content.len() as u64;
    }

    Ok(total_bytes)
}

/// Create a basic test structure for lib tests (simpler version)
pub fn create_lib_test_structure(temp_dir: &TempDir) -> Result<()> {
    let base = temp_dir.path();

    // Create some node_modules directories
    fs::create_dir_all(base.join("project1/node_modules"))?;
    fs::create_dir_all(base.join("project2/node_modules"))?;
    fs::create_dir_all(base.join("nested/project3/node_modules"))?;

    // Create some files to make sure we only delete the right things
    fs::write(base.join("project1/package.json"), "{}")?;
    fs::write(base.join("project1/node_modules/package.json"), "{}")?;

    Ok(())
}

