//! Configuration file loading and saving

use super::{ConfigValidator, Repository};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repositories: Vec<Repository>,
    #[serde(default)]
    pub recipes: Vec<Recipe>,
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

        // Fix indentation for yamllint compliance
        // yamllint expects array items to be indented relative to their parent
        let fixed_yaml = Self::fix_yaml_indentation(&yaml);

        // Make yamllint compliant by adding document separator and ensuring newline at end
        let yaml_content = format!("---\n{}\n", fixed_yaml);

        std::fs::write(path, yaml_content)?;

        Ok(())
    }

    /// Fix YAML indentation to be yamllint compliant
    fn fix_yaml_indentation(yaml: &str) -> String {
        let lines: Vec<&str> = yaml.lines().collect();
        let mut result = Vec::new();

        for line in lines {
            // If line starts with a dash (array item), indent it by 2 spaces
            if line.starts_with("- ") {
                result.push(format!("  {}", line));
            } else if line.starts_with("  ") && !line.starts_with("    ") {
                // If line is already indented by 2 spaces but not 4, make it 4 spaces
                // This handles the properties of array items
                result.push(format!("  {}", line));
            } else {
                result.push(line.to_string());
            }
        }

        result.join("\n")
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
            recipes: Vec::new(),
        }
    }

    /// Find a recipe by name
    pub fn find_recipe(&self, name: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.name == name)
    }

    /// Alias for load method for backwards compatibility
    pub fn load_config(path: &str) -> Result<Self> {
        Self::load(path)
    }

    /// Filter repositories by tag (alias for backwards compatibility)
    pub fn filter_repositories_by_tag(&self, tag: Option<&str>) -> Vec<Repository> {
        self.filter_by_tag(tag)
    }

    /// Filter repositories by context (combining tag inclusion, exclusion, and names filters)
    pub fn filter_repositories(
        &self,
        include_tags: &[String],
        exclude_tags: &[String],
        repos: Option<&[String]>,
    ) -> Vec<Repository> {
        let base_repos = if let Some(repo_names) = repos {
            // If specific repos are specified, filter by names first
            self.filter_by_names(repo_names)
        } else {
            // Otherwise start with all repositories
            self.repositories.clone()
        };

        // Apply both inclusion and exclusion filters in a single pass
        base_repos
            .into_iter()
            .filter(|repo| {
                // Check inclusion filter: if include_tags is empty, include all; otherwise check if repo has any included tag
                let included =
                    include_tags.is_empty() || include_tags.iter().any(|tag| repo.has_tag(tag));

                // Check exclusion filter: if exclude_tags is empty, exclude none; otherwise check if repo has any excluded tag
                let excluded =
                    !exclude_tags.is_empty() && exclude_tags.iter().any(|tag| repo.has_tag(tag));

                included && !excluded
            })
            .collect()
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
            recipes: Vec::new(),
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
        let filtered = config.filter_repositories(
            &["frontend".to_string()],
            &[],
            Some(&["repo1".to_string()]),
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with tag and repo names that don't match
        let filtered =
            config.filter_repositories(&["backend".to_string()], &[], Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 0); // repo1 doesn't have backend tag

        // Test with only repo names
        let filtered = config.filter_repositories(&[], &[], Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with only tag
        let filtered = config.filter_repositories(&["frontend".to_string()], &[], None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with neither (should return all)
        let filtered = config.filter_repositories(&[], &[], None);
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
        let filtered = config.filter_repositories(
            &["nonexistent".to_string()],
            &[],
            Some(&["repo1".to_string()]),
        );
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_repositories_with_valid_tag_and_nonexistent_names() {
        let config = create_test_config();

        // Valid tag with non-existent names should return empty
        let filtered = config.filter_repositories(
            &["backend".to_string()],
            &[],
            Some(&["nonexistent".to_string()]),
        );
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
