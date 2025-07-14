//! Configuration file loading and saving

use super::{ConfigValidator, Repository};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repositories: Vec<Repository>,
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let mut config: Config = serde_yaml::from_str(&content)?;

        // Set the config directory for each repository
        let config_path = Path::new(path);
        let config_dir = config_path.parent().map(|p| p.to_path_buf());

        for repo in &mut config.repositories {
            repo.set_config_dir(config_dir.clone());
        }

        // Validate the loaded configuration
        ConfigValidator::validate_repositories(&config.repositories)?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;

        std::fs::write(path, yaml)?;

        Ok(())
    }

    /// Filter repositories by specific names
    pub fn filter_by_names(&self, names: &[String]) -> Vec<Repository> {
        if names.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|repo| names.contains(&repo.name))
            .cloned()
            .collect()
    }

    /// Filter repositories by tag
    pub fn filter_by_tag(&self, tag: Option<&str>) -> Vec<Repository> {
        match tag {
            Some(tag) => self
                .repositories
                .iter()
                .filter(|repo| repo.has_tag(tag))
                .cloned()
                .collect(),
            None => self.repositories.clone(),
        }
    }

    /// Filter repositories by multiple tags (OR logic)
    pub fn filter_by_any_tag(&self, tags: &[String]) -> Vec<Repository> {
        if tags.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|repo| repo.has_any_tag(tags))
            .cloned()
            .collect()
    }

    /// Filter repositories by multiple tags (AND logic)
    pub fn filter_by_all_tags(&self, tags: &[String]) -> Vec<Repository> {
        if tags.is_empty() {
            return self.repositories.clone();
        }

        self.repositories
            .iter()
            .filter(|repo| tags.iter().all(|tag| repo.has_tag(tag)))
            .cloned()
            .collect()
    }

    /// Get repository by name
    pub fn get_repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.iter().find(|repo| repo.name == name)
    }

    /// Get mutable repository by name
    pub fn get_repository_mut(&mut self, name: &str) -> Option<&mut Repository> {
        self.repositories.iter_mut().find(|repo| repo.name == name)
    }

    /// Add a repository to the configuration
    pub fn add_repository(&mut self, repo: Repository) -> Result<()> {
        // Check for duplicate names
        if self.get_repository(&repo.name).is_some() {
            return Err(anyhow::anyhow!("Repository '{}' already exists", repo.name));
        }

        // Validate the repository
        repo.validate()?;

        self.repositories.push(repo);
        Ok(())
    }

    /// Remove a repository from the configuration
    pub fn remove_repository(&mut self, name: &str) -> bool {
        let initial_len = self.repositories.len();
        self.repositories.retain(|repo| repo.name != name);
        self.repositories.len() != initial_len
    }

    /// Get all unique tags across all repositories
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .repositories
            .iter()
            .flat_map(|repo| repo.tags.iter())
            .cloned()
            .collect();

        tags.sort();
        tags.dedup();
        tags
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        ConfigValidator::validate_repositories(&self.repositories)?;
        Ok(())
    }

    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Alias for load method for backwards compatibility
    pub fn load_config(path: &str) -> Result<Self> {
        Self::load(path)
    }

    /// Filter repositories by tag (alias for backwards compatibility)
    pub fn filter_repositories_by_tag(&self, tag: Option<&str>) -> Vec<Repository> {
        self.filter_by_tag(tag)
    }

    /// Filter repositories by context (combining tag and names filters)
    pub fn filter_repositories(
        &self,
        tag: Option<&str>,
        repos: Option<&[String]>,
    ) -> Vec<Repository> {
        match (tag, repos) {
            // If specific repos are specified, filter by names first, then by tag if provided
            (Some(tag), Some(repo_names)) => {
                let by_names = self.filter_by_names(repo_names);
                by_names
                    .into_iter()
                    .filter(|repo| repo.has_tag(tag))
                    .collect()
            }
            // If only repos are specified, filter by names only
            (None, Some(repo_names)) => self.filter_by_names(repo_names),
            // If only tag is specified, filter by tag only
            (Some(tag), None) => self.filter_by_tag(Some(tag)),
            // If neither is specified, return all repositories
            (None, None) => self.repositories.clone(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let mut repo1 = Repository::new(
            "repo1".to_string(),
            "git@github.com:owner/repo1.git".to_string(),
        );
        repo1.add_tag("frontend".to_string());
        repo1.add_tag("web".to_string());

        let mut repo2 = Repository::new(
            "repo2".to_string(),
            "git@github.com:owner/repo2.git".to_string(),
        );
        repo2.add_tag("backend".to_string());
        repo2.add_tag("api".to_string());

        Config {
            repositories: vec![repo1, repo2],
        }
    }

    #[test]
    fn test_filter_by_tag() {
        let config = create_test_config();

        let frontend_repos = config.filter_by_tag(Some("frontend"));
        assert_eq!(frontend_repos.len(), 1);
        assert_eq!(frontend_repos[0].name, "repo1");

        let all_repos = config.filter_by_tag(None);
        assert_eq!(all_repos.len(), 2);
    }

    #[test]
    fn test_filter_by_any_tag() {
        let config = create_test_config();

        let web_repos = config.filter_by_any_tag(&["frontend".to_string(), "api".to_string()]);
        assert_eq!(web_repos.len(), 2); // Both repos match

        let no_match = config.filter_by_any_tag(&["mobile".to_string()]);
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_get_all_tags() {
        let config = create_test_config();
        let tags = config.get_all_tags();

        assert_eq!(tags, vec!["api", "backend", "frontend", "web"]);
    }

    #[test]
    fn test_filter_by_names() {
        let config = create_test_config();

        let specific_repos = config.filter_by_names(&["repo1".to_string()]);
        assert_eq!(specific_repos.len(), 1);
        assert_eq!(specific_repos[0].name, "repo1");

        let multiple_repos = config.filter_by_names(&["repo1".to_string(), "repo2".to_string()]);
        assert_eq!(multiple_repos.len(), 2);

        let no_match = config.filter_by_names(&["nonexistent".to_string()]);
        assert_eq!(no_match.len(), 0);

        let empty_filter = config.filter_by_names(&[]);
        assert_eq!(empty_filter.len(), 2); // Should return all repos
    }

    #[test]
    fn test_filter_repositories_combined() {
        let config = create_test_config();

        // Test with both tag and repo names
        let filtered = config.filter_repositories(Some("frontend"), Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with tag and repo names that don't match
        let filtered = config.filter_repositories(Some("backend"), Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 0); // repo1 doesn't have backend tag

        // Test with only repo names
        let filtered = config.filter_repositories(None, Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with only tag
        let filtered = config.filter_repositories(Some("frontend"), None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with neither (should return all)
        let filtered = config.filter_repositories(None, None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_add_remove_repository() {
        let mut config = Config::new();

        let repo = Repository::new(
            "test".to_string(),
            "git@github.com:owner/test.git".to_string(),
        );

        config.add_repository(repo).unwrap();
        assert_eq!(config.repositories.len(), 1);

        let removed = config.remove_repository("test");
        assert!(removed);
        assert_eq!(config.repositories.len(), 0);

        let not_removed = config.remove_repository("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_filter_by_empty_names_list() {
        let config = create_test_config();

        // Empty names list should return all repositories
        let filtered = config.filter_by_names(&[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_nonexistent_names() {
        let config = create_test_config();

        // Non-existent names should return empty list
        let filtered =
            config.filter_by_names(&["nonexistent1".to_string(), "nonexistent2".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_partial_match_names() {
        let config = create_test_config();

        // Mix of existing and non-existing names
        let filtered = config.filter_by_names(&["repo1".to_string(), "nonexistent".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");
    }

    #[test]
    fn test_filter_by_nonexistent_tag() {
        let config = create_test_config();

        // Non-existent tag should return empty list
        let filtered = config.filter_by_tag(Some("nonexistent"));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_case_sensitive_tag() {
        let config = create_test_config();

        // Tag filtering should be case sensitive
        let filtered = config.filter_by_tag(Some("BACKEND"));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_repositories_with_nonexistent_tag_and_names() {
        let config = create_test_config();

        // Non-existent tag with valid names should return empty
        let filtered =
            config.filter_repositories(Some("nonexistent"), Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_repositories_with_valid_tag_and_nonexistent_names() {
        let config = create_test_config();

        // Valid tag with non-existent names should return empty
        let filtered =
            config.filter_repositories(Some("backend"), Some(&["nonexistent".to_string()]));
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_any_tag_with_empty_list() {
        let config = create_test_config();

        // Empty tag list should return all repositories
        let filtered = config.filter_by_any_tag(&[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_all_tags_with_empty_list() {
        let config = create_test_config();

        // Empty tag list should return all repositories
        let filtered = config.filter_by_all_tags(&[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_any_tag_with_nonexistent_tags() {
        let config = create_test_config();

        // Non-existent tags should return empty
        let filtered =
            config.filter_by_any_tag(&["nonexistent1".to_string(), "nonexistent2".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_by_all_tags_impossible_combination() {
        let config = create_test_config();

        // Tags that can't all exist on same repo should return empty
        let filtered = config.filter_by_all_tags(&["backend".to_string(), "frontend".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_get_repository_case_sensitive() {
        let config = create_test_config();

        // Repository lookup should be case sensitive
        let repo = config.get_repository("REPO1");
        assert!(repo.is_none());

        let repo = config.get_repository("repo1");
        assert!(repo.is_some());
    }

    #[test]
    fn test_add_repository_duplicate_name() {
        let mut config = create_test_config();

        let duplicate_repo = Repository::new(
            "repo1".to_string(), // Same name as existing
            "git@github.com:other/repo.git".to_string(),
        );

        let result = config.add_repository(duplicate_repo);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_remove_repository_case_sensitive() {
        let mut config = create_test_config();

        // Should not remove with wrong case
        let removed = config.remove_repository("REPO1");
        assert!(!removed);
        assert_eq!(config.repositories.len(), 2);

        // Should remove with correct case
        let removed = config.remove_repository("repo1");
        assert!(removed);
        assert_eq!(config.repositories.len(), 1);
    }
}
