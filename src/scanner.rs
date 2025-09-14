//! Directory scanning functionality for finding node_modules directories

use anyhow::Result;
use glob::Pattern;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scanner for finding node_modules directories
pub struct Scanner {
    root_path: PathBuf,
    exclude_patterns: Vec<Pattern>,
}

impl Scanner {
    /// Create a new scanner with the given root path and exclusion patterns
    pub fn new<P: AsRef<Path>>(root_path: P, exclude_patterns: &[String]) -> Self {
        let compiled_patterns = exclude_patterns
            .iter()
            .filter_map(|pattern| {
                Pattern::new(pattern)
                    .map_err(|e| eprintln!("Warning: Invalid pattern '{}': {}", pattern, e))
                    .ok()
            })
            .collect();

        Self {
            root_path: root_path.as_ref().to_path_buf(),
            exclude_patterns: compiled_patterns,
        }
    }

    /// Find all node_modules directories, applying exclusion filters
    pub fn find_node_modules_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut targets = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_entry(|e| {
                // Don't traverse into directories if we're already inside a node_modules directory
                // Check if any parent in the path is named "node_modules"
                let components: Vec<&str> = e.path().components()
                    .filter_map(|c| c.as_os_str().to_str())
                    .collect();

                // If we find "node_modules" in the path components,
                // don't traverse deeper unless this is the node_modules directory itself
                if let Some(node_modules_index) = components.iter().position(|&c| c == "node_modules") {
                    // If this entry is the node_modules directory itself, allow it
                    // But don't traverse into it
                    e.file_name() != "node_modules" || components.len() == node_modules_index + 1
                } else {
                    // No node_modules in path, allow traversal
                    true
                }
            })
        {
            let entry = entry?;
            let path = entry.path();

            // Check if this is a node_modules directory
            if entry.file_type().is_dir() && path.file_name() == Some("node_modules".as_ref()) {
                // Apply exclusion filters
                if !self.should_exclude(path) {
                    targets.push(path.to_path_buf());
                }
            }
        }

        // Sort for consistent ordering
        targets.sort();
        Ok(targets)
    }

    /// Check if a path should be excluded based on the exclusion patterns
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.exclude_patterns {
            if pattern.matches(&path_str) {
                return true;
            }
        }

        false
    }

    /// Get a preview of directories that would be affected (for display purposes)
    pub fn get_parent_directories(&self, paths: &[PathBuf]) -> Vec<PathBuf> {
        paths
            .iter()
            .filter_map(|path| path.parent().map(|p| p.to_path_buf()))
            .collect()
    }

    /// Get the number of compiled exclusion patterns (for testing)
    pub fn exclusion_pattern_count(&self) -> usize {
        self.exclude_patterns.len()
    }
}

/// Validate that all paths end with "node_modules" for safety
pub fn validate_targets(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        if path.file_name() != Some("node_modules".as_ref()) {
            return Err(anyhow::anyhow!(
                "Safety check failed: path '{}' does not end with 'node_modules'",
                path.display()
            ));
        }
    }
    Ok(())
}