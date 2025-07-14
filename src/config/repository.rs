//! Repository configuration and utilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip)]
    pub config_dir: Option<PathBuf>,
}

impl Repository {
    /// Create a new repository configuration
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            tags: Vec::new(),
            path: None,
            branch: None,
            config_dir: None,
        }
    }

    /// Check if repository has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if repository has any of the specified tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|tag| self.has_tag(tag))
    }

    /// Check if the repository URL has a valid format
    pub fn is_url_valid(&self) -> bool {
        self.url.starts_with("git@")
            || self.url.starts_with("https://")
            || self.url.starts_with("http://")
    }

    /// Validate repository configuration
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Repository name cannot be empty"));
        }

        if self.url.is_empty() {
            return Err(anyhow::anyhow!("Repository URL cannot be empty"));
        }

        if !self.is_url_valid() {
            return Err(anyhow::anyhow!("Invalid repository URL: {}", self.url));
        }

        Ok(())
    }

    /// Get the target directory for cloning
    pub fn get_target_dir(&self) -> String {
        match &self.path {
            Some(path) => {
                let path_buf = PathBuf::from(path);
                if path_buf.is_absolute() {
                    // If it's already an absolute path, use it as-is
                    path.clone()
                } else {
                    // If it's a relative path, resolve it relative to the config file directory
                    if let Some(config_dir) = &self.config_dir {
                        config_dir.join(path).to_string_lossy().to_string()
                    } else {
                        // Fallback to current directory if config_dir is not available
                        std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."))
                            .join(path)
                            .to_string_lossy()
                            .to_string()
                    }
                }
            }
            None => {
                // Default to repository name as relative path
                if let Some(config_dir) = &self.config_dir {
                    config_dir.join(&self.name).to_string_lossy().to_string()
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(&self.name)
                        .to_string_lossy()
                        .to_string()
                }
            }
        }
    }

    /// Set the configuration directory (used by config loader)
    pub fn set_config_dir(&mut self, config_dir: Option<PathBuf>) {
        self.config_dir = config_dir;
    }

    /// Add a tag to the repository
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the repository
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Check if the repository directory exists
    pub fn exists(&self) -> bool {
        Path::new(&self.get_target_dir()).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_relative_path_resolution() {
        let mut repo = Repository {
            name: "test-repo".to_string(),
            url: "git@github.com:owner/repo.git".to_string(),
            tags: vec![],
            path: Some("journey".to_string()),
            branch: None,
            config_dir: Some(PathBuf::from("/some/config/dir")),
        };

        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/some/config/dir/journey");

        // Test absolute path (should use as-is)
        repo.path = Some("/absolute/path/to/repo".to_string());
        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/absolute/path/to/repo");

        // Test default path when no path is specified
        repo.path = None;
        let target_dir = repo.get_target_dir();
        assert_eq!(target_dir, "/some/config/dir/test-repo");
    }

    #[test]
    fn test_no_config_dir_fallback() {
        let current_dir = env::current_dir().unwrap();

        let repo = Repository {
            name: "test-repo".to_string(),
            url: "git@github.com:owner/repo.git".to_string(),
            tags: vec![],
            path: Some("journey".to_string()),
            branch: None,
            config_dir: None,
        };

        let target_dir = repo.get_target_dir();
        let expected = current_dir.join("journey").to_string_lossy().to_string();
        assert_eq!(target_dir, expected);
    }

    #[test]
    fn test_url_validation() {
        let repo_ssh = Repository::new(
            "test".to_string(),
            "git@github.com:owner/repo.git".to_string(),
        );
        assert!(repo_ssh.is_url_valid());

        let repo_https = Repository::new(
            "test".to_string(),
            "https://github.com/owner/repo.git".to_string(),
        );
        assert!(repo_https.is_url_valid());

        let repo_invalid = Repository::new("test".to_string(), "invalid-url".to_string());
        assert!(!repo_invalid.is_url_valid());
    }

    #[test]
    fn test_tag_operations() {
        let mut repo = Repository::new(
            "test".to_string(),
            "git@github.com:owner/repo.git".to_string(),
        );

        assert!(!repo.has_tag("frontend"));

        repo.add_tag("frontend".to_string());
        assert!(repo.has_tag("frontend"));

        repo.add_tag("backend".to_string());
        assert!(repo.has_any_tag(&["frontend".to_string()]));
        assert!(repo.has_any_tag(&["backend".to_string()]));
        assert!(!repo.has_any_tag(&["mobile".to_string()]));

        repo.remove_tag("frontend");
        assert!(!repo.has_tag("frontend"));
        assert!(repo.has_tag("backend"));
    }

    #[test]
    fn test_default_path_resolution() {
        // Test repository without explicit path
        let repo_without_path = Repository::new(
            "my-repo".to_string(),
            "git@github.com:owner/my-repo.git".to_string(),
        );

        // Should default to repository name
        let target_dir = repo_without_path.get_target_dir();
        assert!(target_dir.ends_with("my-repo"));

        // Test repository with explicit path
        let mut repo_with_path = Repository::new(
            "my-repo".to_string(),
            "git@github.com:owner/my-repo.git".to_string(),
        );
        repo_with_path.path = Some("custom/path".to_string());

        let target_dir_custom = repo_with_path.get_target_dir();
        assert!(target_dir_custom.ends_with("custom/path"));

        // Test with config directory set
        let mut repo_with_config_dir = Repository::new(
            "test-repo".to_string(),
            "git@github.com:owner/test-repo.git".to_string(),
        );
        repo_with_config_dir.set_config_dir(Some(std::path::PathBuf::from("/config/dir")));

        let target_dir_with_config = repo_with_config_dir.get_target_dir();
        assert!(target_dir_with_config.contains("/config/dir"));
        assert!(target_dir_with_config.ends_with("test-repo"));
    }

    #[test]
    fn test_validation() {
        let valid_repo = Repository::new(
            "test".to_string(),
            "git@github.com:owner/repo.git".to_string(),
        );
        assert!(valid_repo.validate().is_ok());

        let empty_name =
            Repository::new("".to_string(), "git@github.com:owner/repo.git".to_string());
        assert!(empty_name.validate().is_err());

        let empty_url = Repository::new("test".to_string(), "".to_string());
        assert!(empty_url.validate().is_err());

        let invalid_url = Repository::new("test".to_string(), "invalid-url".to_string());
        assert!(invalid_url.validate().is_err());
    }
}
