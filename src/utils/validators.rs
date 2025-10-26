//! Configuration validation utilities
//!
//! This module provides centralized validation logic for all configuration-related
//! validation rules, promoting separation of concerns and improved testability.

use crate::config::{Config, Recipe, Repository};
use anyhow::{Result, anyhow};
use std::collections::HashSet;

/// Enumeration of possible validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Configuration has an empty repositories list
    EmptyRepositoryList,
    /// Repository name is empty
    EmptyRepositoryName(String),
    /// Repository URL is empty
    EmptyRepositoryUrl(String),
    /// Repository URL format is invalid
    InvalidRepositoryUrl(String, String),
    /// Duplicate repository names found
    DuplicateRepositoryName(String),
    /// Recipe has no steps defined
    RecipeWithNoSteps(String),
    /// Recipe name is empty
    EmptyRecipeName,
    /// Duplicate recipe names found
    DuplicateRecipeName(String),
    /// Tag filter is empty or whitespace-only
    EmptyTagFilter(String),
    /// No repositories found with specified tag
    TagNotFound(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyRepositoryList => {
                write!(f, "Configuration must contain at least one repository")
            }
            ValidationError::EmptyRepositoryName(name) => {
                write!(f, "Repository name cannot be empty: '{}'", name)
            }
            ValidationError::EmptyRepositoryUrl(name) => {
                write!(f, "Repository '{}' URL cannot be empty", name)
            }
            ValidationError::InvalidRepositoryUrl(name, url) => {
                write!(f, "Repository '{}' has invalid URL: '{}'", name, url)
            }
            ValidationError::DuplicateRepositoryName(name) => {
                write!(f, "Duplicate repository name: '{}'", name)
            }
            ValidationError::RecipeWithNoSteps(name) => {
                write!(f, "Recipe '{}' must contain at least one step", name)
            }
            ValidationError::EmptyRecipeName => {
                write!(f, "Recipe name cannot be empty")
            }
            ValidationError::DuplicateRecipeName(name) => {
                write!(f, "Duplicate recipe name: '{}'", name)
            }
            ValidationError::EmptyTagFilter(filter) => {
                write!(f, "Tag filter cannot be empty: '{}'", filter)
            }
            ValidationError::TagNotFound(tag) => {
                write!(f, "No repositories found with tag: '{}'", tag)
            }
        }
    }
}

/// Validates a complete configuration object
///
/// This function performs comprehensive validation of the entire configuration,
/// including repositories and recipes.
pub fn validate_config(config: &Config) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate repositories
    if let Err(mut repo_errors) = validate_repositories(&config.repositories) {
        errors.append(&mut repo_errors);
    }

    // Validate recipes
    if let Err(mut recipe_errors) = validate_recipes(&config.recipes) {
        errors.append(&mut recipe_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a list of repositories
///
/// Checks for duplicate names and validates each individual repository.
pub fn validate_repositories(repositories: &[Repository]) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check for duplicate names
    let mut names = HashSet::new();
    for repo in repositories {
        if !names.insert(&repo.name) {
            errors.push(ValidationError::DuplicateRepositoryName(repo.name.clone()));
        }
    }

    // Validate each repository individually
    for repo in repositories {
        if let Err(mut repo_errors) = validate_repository(repo) {
            errors.append(&mut repo_errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a single repository
///
/// Checks that the repository has a valid name and URL.
pub fn validate_repository(repository: &Repository) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check for empty name
    if repository.name.is_empty() {
        errors.push(ValidationError::EmptyRepositoryName(
            repository.name.clone(),
        ));
    }

    // Check for empty URL
    if repository.url.is_empty() {
        errors.push(ValidationError::EmptyRepositoryUrl(repository.name.clone()));
    } else if !is_valid_repository_url(&repository.url) {
        // Check URL format only if URL is not empty
        errors.push(ValidationError::InvalidRepositoryUrl(
            repository.name.clone(),
            repository.url.clone(),
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a list of recipes
///
/// Checks for duplicate names and validates each individual recipe.
pub fn validate_recipes(recipes: &[Recipe]) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check for duplicate names
    let mut names = HashSet::new();
    for recipe in recipes {
        if !names.insert(&recipe.name) {
            errors.push(ValidationError::DuplicateRecipeName(recipe.name.clone()));
        }
    }

    // Validate each recipe individually
    for recipe in recipes {
        if let Err(mut recipe_errors) = validate_recipe(recipe) {
            errors.append(&mut recipe_errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a single recipe
///
/// Checks that the recipe has a name and at least one step.
pub fn validate_recipe(recipe: &Recipe) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check for empty name
    if recipe.name.is_empty() {
        errors.push(ValidationError::EmptyRecipeName);
    }

    // Check for empty steps
    if recipe.steps.is_empty() {
        errors.push(ValidationError::RecipeWithNoSteps(recipe.name.clone()));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates a tag filter string
///
/// Ensures the tag filter is not empty or whitespace-only.
pub fn validate_tag_filter(filter: &str) -> Result<(), ValidationError> {
    if filter.trim().is_empty() {
        Err(ValidationError::EmptyTagFilter(filter.to_string()))
    } else {
        Ok(())
    }
}

/// Validates that a tag exists in the given repositories
///
/// Checks if at least one repository has the specified tag.
pub fn validate_tag_exists(repositories: &[Repository], tag: &str) -> Result<(), ValidationError> {
    let has_tag = repositories.iter().any(|repo| repo.has_tag(tag));

    if has_tag {
        Ok(())
    } else {
        Err(ValidationError::TagNotFound(tag.to_string()))
    }
}

/// Helper function to check if a repository URL is valid
///
/// Validates common Git URL formats (SSH, HTTPS, HTTP).
fn is_valid_repository_url(url: &str) -> bool {
    url.starts_with("git@") || url.starts_with("https://") || url.starts_with("http://")
}

/// Converts validation errors to a user-friendly anyhow error
///
/// This helper function is useful for maintaining backward compatibility
/// with existing error handling patterns.
pub fn validation_errors_to_anyhow(errors: Vec<ValidationError>) -> anyhow::Error {
    let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    anyhow!("Validation errors: {}", error_messages.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Repository;

    fn create_valid_repository(name: &str, url: &str) -> Repository {
        Repository::new(name.to_string(), url.to_string())
    }

    fn create_valid_recipe(name: &str, steps: Vec<&str>) -> Recipe {
        Recipe {
            name: name.to_string(),
            steps: steps.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_validate_config_empty_repositories() {
        let config = Config {
            repositories: vec![],
            recipes: vec![],
        };

        // Empty repositories should be allowed (config can be initialized empty)
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            repositories: vec![create_valid_repository(
                "repo1",
                "git@github.com:owner/repo1.git",
            )],
            recipes: vec![create_valid_recipe("recipe1", vec!["echo hello"])],
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_repositories_valid() {
        let repos = vec![
            create_valid_repository("repo1", "git@github.com:owner/repo1.git"),
            create_valid_repository("repo2", "https://github.com/owner/repo2.git"),
        ];

        assert!(validate_repositories(&repos).is_ok());
    }

    #[test]
    fn test_validate_repositories_duplicate_names() {
        let repos = vec![
            create_valid_repository("repo1", "git@github.com:owner/repo1.git"),
            create_valid_repository("repo1", "git@github.com:owner/repo2.git"),
        ];

        let result = validate_repositories(&repos);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ValidationError::DuplicateRepositoryName(_)
        ));
    }

    #[test]
    fn test_validate_repository_empty_name() {
        let repo = Repository::new("".to_string(), "git@github.com:owner/repo.git".to_string());

        let result = validate_repository(&repo);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::EmptyRepositoryName(_)))
        );
    }

    #[test]
    fn test_validate_repository_empty_url() {
        let repo = Repository::new("repo1".to_string(), "".to_string());

        let result = validate_repository(&repo);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::EmptyRepositoryUrl(_)))
        );
    }

    #[test]
    fn test_validate_repository_invalid_url() {
        let repo = Repository::new("repo1".to_string(), "invalid-url".to_string());

        let result = validate_repository(&repo);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::InvalidRepositoryUrl(_, _)))
        );
    }

    #[test]
    fn test_validate_recipes_valid() {
        let recipes = vec![
            create_valid_recipe("recipe1", vec!["echo hello"]),
            create_valid_recipe("recipe2", vec!["echo world", "ls -la"]),
        ];

        assert!(validate_recipes(&recipes).is_ok());
    }

    #[test]
    fn test_validate_recipes_duplicate_names() {
        let recipes = vec![
            create_valid_recipe("recipe1", vec!["echo hello"]),
            create_valid_recipe("recipe1", vec!["echo world"]),
        ];

        let result = validate_recipes(&recipes);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::DuplicateRecipeName(_)));
    }

    #[test]
    fn test_validate_recipe_empty_name() {
        let recipe = Recipe {
            name: "".to_string(),
            steps: vec!["echo hello".to_string()],
        };

        let result = validate_recipe(&recipe);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::EmptyRecipeName))
        );
    }

    #[test]
    fn test_validate_recipe_no_steps() {
        let recipe = Recipe {
            name: "recipe1".to_string(),
            steps: vec![],
        };

        let result = validate_recipe(&recipe);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::RecipeWithNoSteps(_)))
        );
    }

    #[test]
    fn test_validate_tag_filter_valid() {
        assert!(validate_tag_filter("frontend").is_ok());
        assert!(validate_tag_filter("backend").is_ok());
        assert!(validate_tag_filter("my-tag").is_ok());
    }

    #[test]
    fn test_validate_tag_filter_empty() {
        assert!(validate_tag_filter("").is_err());
        assert!(validate_tag_filter("   ").is_err());
        assert!(validate_tag_filter("\t\n").is_err());
    }

    #[test]
    fn test_validate_tag_exists() {
        let mut repo1 = create_valid_repository("repo1", "git@github.com:owner/repo1.git");
        repo1.add_tag("frontend".to_string());

        let mut repo2 = create_valid_repository("repo2", "git@github.com:owner/repo2.git");
        repo2.add_tag("backend".to_string());

        let repos = vec![repo1, repo2];

        assert!(validate_tag_exists(&repos, "frontend").is_ok());
        assert!(validate_tag_exists(&repos, "backend").is_ok());
        assert!(validate_tag_exists(&repos, "nonexistent").is_err());
    }

    #[test]
    fn test_is_valid_repository_url() {
        assert!(is_valid_repository_url("git@github.com:owner/repo.git"));
        assert!(is_valid_repository_url("https://github.com/owner/repo.git"));
        assert!(is_valid_repository_url("http://github.com/owner/repo.git"));
        assert!(!is_valid_repository_url("invalid-url"));
        assert!(!is_valid_repository_url("ftp://example.com/repo.git"));
    }

    #[test]
    fn test_validation_errors_to_anyhow() {
        let errors = vec![
            ValidationError::EmptyRepositoryName("test".to_string()),
            ValidationError::RecipeWithNoSteps("recipe1".to_string()),
        ];

        let anyhow_error = validation_errors_to_anyhow(errors);
        let error_message = format!("{}", anyhow_error);

        assert!(error_message.contains("Repository name cannot be empty"));
        assert!(error_message.contains("Recipe 'recipe1' must contain at least one step"));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::DuplicateRepositoryName("test-repo".to_string());
        assert_eq!(
            format!("{}", error),
            "Duplicate repository name: 'test-repo'"
        );

        let error = ValidationError::RecipeWithNoSteps("test-recipe".to_string());
        assert_eq!(
            format!("{}", error),
            "Recipe 'test-recipe' must contain at least one step"
        );
    }
}
