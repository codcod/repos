//! List command implementation

use super::{Command, CommandContext};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use serde::Serialize;

/// Output format for a repository in JSON mode
#[derive(Serialize)]
struct RepositoryOutput {
    name: String,
    url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
}

/// List command for displaying repositories with optional filtering
pub struct ListCommand {
    /// Output in JSON format
    pub json: bool,
}

#[async_trait]
impl Command for ListCommand {
    async fn execute(&self, context: &CommandContext) -> Result<()> {
        let repositories = context.config.filter_repositories(
            &context.tag,
            &context.exclude_tag,
            context.repos.as_deref(),
        );

        if self.json {
            // JSON output mode
            let output: Vec<RepositoryOutput> = repositories
                .iter()
                .map(|repo| RepositoryOutput {
                    name: repo.name.clone(),
                    url: repo.url.clone(),
                    tags: repo.tags.clone(),
                    path: repo.path.clone(),
                    branch: repo.branch.clone(),
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(());
        }

        // Human-readable output mode
        if repositories.is_empty() {
            let mut filter_parts = Vec::new();

            if !context.tag.is_empty() {
                filter_parts.push(format!("tags {:?}", context.tag));
            }
            if !context.exclude_tag.is_empty() {
                filter_parts.push(format!("excluding tags {:?}", context.exclude_tag));
            }
            if let Some(repos) = &context.repos {
                filter_parts.push(format!("repositories {:?}", repos));
            }

            let filter_desc = if filter_parts.is_empty() {
                "no repositories found".to_string()
            } else {
                filter_parts.join(" and ")
            };

            println!(
                "{}",
                format!("No repositories found with {filter_desc}").yellow()
            );
            return Ok(());
        }

        // Print summary header
        println!(
            "{}",
            format!("Found {} repositories", repositories.len()).green()
        );
        println!();

        // Print each repository
        for repo in &repositories {
            println!("{} {}", "•".blue(), repo.name.bold());
            println!("  URL: {}", repo.url);

            if !repo.tags.is_empty() {
                println!("  Tags: {}", repo.tags.join(", ").cyan());
            }

            if let Some(path) = &repo.path {
                println!("  Path: {}", path);
            }

            if let Some(branch) = &repo.branch {
                println!("  Branch: {}", branch);
            }

            println!();
        }

        // Print summary footer
        println!(
            "{}",
            format!("Total: {} repositories", repositories.len()).green()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Repository};

    /// Helper function to create a test config with repositories
    fn create_test_config() -> Config {
        let mut repo1 = Repository::new(
            "test-repo-1".to_string(),
            "https://github.com/test/repo1.git".to_string(),
        );
        repo1.tags = vec!["frontend".to_string(), "javascript".to_string()];

        let mut repo2 = Repository::new(
            "test-repo-2".to_string(),
            "https://github.com/test/repo2.git".to_string(),
        );
        repo2.tags = vec!["backend".to_string(), "rust".to_string()];

        let mut repo3 = Repository::new(
            "test-repo-3".to_string(),
            "https://github.com/test/repo3.git".to_string(),
        );
        repo3.tags = vec!["frontend".to_string(), "typescript".to_string()];

        Config {
            repositories: vec![repo1, repo2, repo3],
            recipes: vec![],
        }
    }

    /// Helper to create CommandContext for testing
    fn create_context(
        config: Config,
        tag: Vec<String>,
        exclude_tag: Vec<String>,
        repos: Option<Vec<String>>,
    ) -> CommandContext {
        CommandContext {
            config,
            tag,
            exclude_tag,
            repos,
            parallel: false,
        }
    }

    #[tokio::test]
    async fn test_list_command_all_repositories() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(config, vec![], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_with_tag_filter() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(config, vec!["frontend".to_string()], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_with_exclude_tag() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(config, vec![], vec!["backend".to_string()], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_with_both_filters() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(
            config,
            vec!["frontend".to_string()],
            vec!["javascript".to_string()],
            None,
        );

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_no_matches() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(config, vec!["nonexistent".to_string()], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_with_repo_filter() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(
            config,
            vec![],
            vec![],
            Some(vec!["test-repo-1".to_string(), "test-repo-2".to_string()]),
        );

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_empty_config() {
        let config = Config {
            repositories: vec![],
            recipes: vec![],
        };
        let command = ListCommand { json: false };

        let context = create_context(config, vec![], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_multiple_tags() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(
            config,
            vec!["frontend".to_string(), "rust".to_string()],
            vec![],
            None,
        );

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_combined_filters() {
        let config = create_test_config();
        let command = ListCommand { json: false };

        let context = create_context(
            config,
            vec!["frontend".to_string()],
            vec![],
            Some(vec!["test-repo-1".to_string()]),
        );

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_json_output() {
        let config = create_test_config();
        let command = ListCommand { json: true };

        let context = create_context(config, vec![], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_json_with_filters() {
        let config = create_test_config();
        let command = ListCommand { json: true };

        let context = create_context(config, vec!["frontend".to_string()], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_command_json_empty() {
        let config = Config {
            repositories: vec![],
            recipes: vec![],
        };
        let command = ListCommand { json: true };

        let context = create_context(config, vec![], vec![], None);

        let result = command.execute(&context).await;
        assert!(result.is_ok());
    }
}
