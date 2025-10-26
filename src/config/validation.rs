//! Configuration validation utilities
//!
//! This module provides backward compatibility wrappers for the centralized
//! validation logic in utils::validators.

use super::Repository;
use crate::utils::validators;
use anyhow::Result;

/// Configuration validator
///
/// This struct provides backward compatibility with existing validation patterns
/// while delegating to the centralized utils::validators module.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a single repository configuration
    pub fn validate_repository(repo: &Repository) -> Result<()> {
        validators::validate_repository(repo)
            .map_err(|errors| validators::validation_errors_to_anyhow(errors))
    }

    /// Validate multiple repositories
    pub fn validate_repositories(repos: &[Repository]) -> Result<()> {
        validators::validate_repositories(repos)
            .map_err(|errors| validators::validation_errors_to_anyhow(errors))
    }

    /// Validate tag filters
    pub fn validate_tag_filter(filter: &str) -> Result<()> {
        validators::validate_tag_filter(filter).map_err(|error| anyhow::anyhow!("{}", error))
    }

    /// Check if all repositories with the given tag exist
    pub fn validate_tag_exists(repos: &[Repository], tag: &str) -> Result<()> {
        validators::validate_tag_exists(repos, tag).map_err(|error| anyhow::anyhow!("{}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_repositories() {
        let repos = vec![
            Repository::new(
                "repo1".to_string(),
                "git@github.com:owner/repo1.git".to_string(),
            ),
            Repository::new(
                "repo2".to_string(),
                "git@github.com:owner/repo2.git".to_string(),
            ),
        ];

        assert!(validators::validate_repositories(&repos).is_ok());
    }

    #[test]
    fn test_duplicate_names() {
        let repos = vec![
            Repository::new(
                "repo1".to_string(),
                "git@github.com:owner/repo1.git".to_string(),
            ),
            Repository::new(
                "repo1".to_string(),
                "git@github.com:owner/repo2.git".to_string(),
            ),
        ];

        assert!(validators::validate_repositories(&repos).is_err());
    }

    #[test]
    fn test_tag_filter_validation() {
        assert!(validators::validate_tag_filter("frontend").is_ok());
        assert!(validators::validate_tag_filter("").is_err());
        assert!(validators::validate_tag_filter("   ").is_err());
    }

    #[test]
    fn test_tag_exists_validation() {
        let mut repo1 = Repository::new(
            "repo1".to_string(),
            "git@github.com:owner/repo1.git".to_string(),
        );
        repo1.add_tag("frontend".to_string());

        let repos = vec![repo1];

        assert!(validators::validate_tag_exists(&repos, "frontend").is_ok());
        assert!(validators::validate_tag_exists(&repos, "backend").is_err());
    }
}
