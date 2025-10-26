//! File system utility functions

use anyhow::Result;

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory_exists(path: &str) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_directory_exists_new_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_directory");

        assert!(!new_dir.exists());
        ensure_directory_exists(new_dir.to_str().unwrap()).unwrap();
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_ensure_directory_exists_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let existing_dir = temp_dir.path().join("existing");
        fs::create_dir(&existing_dir).unwrap();

        assert!(existing_dir.exists());
        // Should not error on existing directory
        ensure_directory_exists(existing_dir.to_str().unwrap()).unwrap();
        assert!(existing_dir.exists());
    }

    #[test]
    fn test_ensure_directory_exists_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("level1").join("level2").join("level3");

        assert!(!nested_path.exists());
        ensure_directory_exists(nested_path.to_str().unwrap()).unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.is_dir());

        // Check intermediate directories were created
        assert!(temp_dir.path().join("level1").exists());
        assert!(temp_dir.path().join("level1").join("level2").exists());
    }
}
