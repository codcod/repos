//! Configuration validation utilities

use super::Repository;
use anyhow::Result;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a single repository configuration
    pub fn validate_repository(repo: &Repository) -> Result<()> {
        repo.validate()
    }

    /// Validate multiple repositories
    pub fn validate_repositories(repos: &[Repository]) -> Result<()> {
        let mut errors = Vec::new();

        // Check for duplicate names
        let mut names = std::collections::HashSet::new();
        for repo in repos {
            if !names.insert(&repo.name) {
                errors.push(format!("Duplicate repository name: {}", repo.name));
            }
        }

        // Validate each repository
        for repo in repos {
            if let Err(e) = repo.validate() {
                errors.push(format!("Repository '{}': {}", repo.name, e));
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!("Validation errors: {}", errors.join("; ")));
        }

        Ok(())
    }

    /// Validate tag filters
    pub fn validate_tag_filter(filter: &str) -> Result<()> {
        if filter.trim().is_empty() {
            return Err(anyhow::anyhow!("Tag filter cannot be empty: {}", filter));
        }

        // Additional tag filter validation can be added here
        // For example, check for invalid characters, length limits, etc.

        Ok(())
    }

    /// Check if all repositories with the given tag exist
    pub fn validate_tag_exists(repos: &[Repository], tag: &str) -> Result<()> {
        let has_tag = repos.iter().any(|repo| repo.has_tag(tag));

        if !has_tag {
            return Err(anyhow::anyhow!("No repositories found with tag: {}", tag));
        }

        Ok(())
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

        assert!(ConfigValidator::validate_repositories(&repos).is_ok());
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

        assert!(ConfigValidator::validate_repositories(&repos).is_err());
    }

    #[test]
    fn test_tag_filter_validation() {
        assert!(ConfigValidator::validate_tag_filter("frontend").is_ok());
        assert!(ConfigValidator::validate_tag_filter("").is_err());
        assert!(ConfigValidator::validate_tag_filter("   ").is_err());
    }

    #[test]
    fn test_tag_exists_validation() {
        let mut repo1 = Repository::new(
            "repo1".to_string(),
            "git@github.com:owner/repo1.git".to_string(),
        );
        repo1.add_tag("frontend".to_string());

        let repos = vec![repo1];

        assert!(ConfigValidator::validate_tag_exists(&repos, "frontend").is_ok());
        assert!(ConfigValidator::validate_tag_exists(&repos, "backend").is_err());
    }
}
