use anyhow::Result;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// In-memory index of repository files for efficient querying
/// Built with a single filesystem traversal to avoid repeated walks
#[derive(Debug)]
pub struct RepoIndex {
    /// All files in the repository
    pub files: Vec<PathBuf>,
    /// Fast lookup by file name (e.g., "pom.xml")
    pub file_names: HashSet<String>,
    /// Fast lookup by extension (e.g., "java", "kt")
    pub extensions: HashSet<String>,
    /// Map of relative path to full path for quick queries
    #[allow(dead_code)]
    pub path_map: HashMap<PathBuf, PathBuf>,
}

impl RepoIndex {
    /// Build an index by walking the repository once
    pub fn build(root: &Path) -> Result<Self> {
        let mut files = Vec::new();
        let mut file_names = HashSet::new();
        let mut extensions = HashSet::new();
        let mut path_map = HashMap::new();

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();

            if entry.file_type().is_file() {
                files.push(path.clone());

                // Index file name
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    file_names.insert(name.to_string());
                }

                // Index extension
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    extensions.insert(ext.to_string());
                }

                // Index relative path
                if let Ok(rel_path) = path.strip_prefix(root) {
                    path_map.insert(rel_path.to_path_buf(), path.clone());
                }
            }
        }

        Ok(Self {
            files,
            file_names,
            extensions,
            path_map,
        })
    }

    /// Check if any file with the given name exists
    pub fn has_file(&self, name: &str) -> bool {
        self.file_names.contains(name)
    }

    /// Check if any file with the given extension exists
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extensions.contains(ext)
    }

    /// Check if any file contains a pattern in its path
    pub fn has_path_pattern(&self, pattern: &str) -> bool {
        self.files.iter().any(|p| {
            p.to_string_lossy().contains(pattern)
        })
    }

    /// Get all files with a specific extension
    #[allow(dead_code)]
    pub fn files_with_extension(&self, ext: &str) -> Vec<&PathBuf> {
        self.files
            .iter()
            .filter(|path| {
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == ext)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all files with a specific name
    pub fn files_with_name(&self, name: &str) -> Vec<&PathBuf> {
        self.files
            .iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == name)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all files matching any of the extensions
    pub fn files_with_extensions(&self, exts: &[&str]) -> Vec<&PathBuf> {
        self.files
            .iter()
            .filter(|path| {
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| exts.contains(&e))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get files in a specific directory (shallow)
    #[allow(dead_code)]
    pub fn files_in_dir(&self, dir_name: &str) -> Vec<&PathBuf> {
        self.files
            .iter()
            .filter(|path| {
                path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|n| n == dir_name)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all files matching a path pattern
    #[allow(dead_code)]
    pub fn files_matching_pattern(&self, pattern: &str) -> Vec<&PathBuf> {
        self.files
            .iter()
            .filter(|path| path.to_string_lossy().contains(pattern))
            .collect()
    }

    /// Count files with a specific extension
    #[allow(dead_code)]
    pub fn count_extension(&self, ext: &str) -> usize {
        self.files_with_extension(ext).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_repo_index_basic() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create test structure
        fs::write(root.join("pom.xml"), "").unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src").join("Main.java"), "").unwrap();
        fs::write(root.join("src").join("Utils.kt"), "").unwrap();

        let index = RepoIndex::build(root).unwrap();

        assert!(index.has_file("pom.xml"));
        assert!(index.has_extension("java"));
        assert!(index.has_extension("kt"));
        assert_eq!(index.count_extension("java"), 1);
        assert_eq!(index.count_extension("kt"), 1);
    }

    #[test]
    fn test_repo_index_patterns() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("src/main/java")).unwrap();
        fs::write(root.join("src/main/java/App.java"), "").unwrap();

        let index = RepoIndex::build(root).unwrap();

        assert!(index.has_path_pattern("src/main/java"));
        assert_eq!(index.files_matching_pattern("main").len(), 1);
    }
}
