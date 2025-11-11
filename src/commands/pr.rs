//! Pull request command implementation

use super::{Command, CommandContext};
use crate::github::PrOptions;
use crate::github::api::create_pr_from_workspace;
use anyhow::Result;
use async_trait::async_trait;
use colored::*;

/// Pull request command for creating PRs with changes
pub struct PrCommand {
    pub title: String,
    pub body: String,
    pub branch_name: Option<String>,
    pub base_branch: Option<String>,
    pub commit_msg: Option<String>,
    pub draft: bool,
    pub token: String,
    pub create_only: bool,
}

#[async_trait]
impl Command for PrCommand {
    async fn execute(&self, context: &CommandContext) -> Result<()> {
        let repositories = context.config.filter_repositories(
            &context.tag,
            &context.exclude_tag,
            context.repos.as_deref(),
        );

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

        println!(
            "{}",
            format!(
                "Checking {} repositories for changes...",
                repositories.len()
            )
            .green()
        );

        let pr_options = PrOptions {
            title: self.title.clone(),
            body: self.body.clone(),
            branch_name: self.branch_name.clone(),
            base_branch: self.base_branch.clone(),
            commit_msg: self.commit_msg.clone(),
            draft: self.draft,
            token: self.token.clone(),
            create_only: self.create_only,
        };

        let mut errors = Vec::new();
        let mut successful = 0;

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let pr_options = pr_options.clone();
                    async move {
                        (
                            repo.name.clone(),
                            create_pr_from_workspace(&repo, &pr_options).await,
                        )
                    }
                })
                .collect();

            for task in tasks {
                let (repo_name, result) = task.await;
                match result {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        eprintln!("{}", format!("Error: {e}").red());
                        errors.push((repo_name, e));
                    }
                }
            }
        } else {
            for repo in repositories {
                match create_pr_from_workspace(&repo, &pr_options).await {
                    Ok(_) => successful += 1,
                    Err(e) => {
                        eprintln!(
                            "{} | {}",
                            repo.name.cyan().bold(),
                            format!("Error: {e}").red()
                        );
                        errors.push((repo.name.clone(), e));
                    }
                }
            }
        }

        // Report summary
        if errors.is_empty() {
            println!("{}", "Done processing pull requests".green());
        } else {
            println!(
                "{}",
                format!(
                    "Completed with {} successful, {} failed",
                    successful,
                    errors.len()
                )
                .yellow()
            );

            // If all operations failed, return an error to propagate to main
            if successful == 0 {
                return Err(anyhow::anyhow!(
                    "All pull request operations failed. First error: {}",
                    errors[0].1
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Repository};

    #[tokio::test]
    async fn test_pr_command_no_repositories() {
        let config = Config {
            repositories: vec![],
            recipes: vec![],
        };
        let context = CommandContext {
            config,
            tag: vec![],
            exclude_tag: vec![],
            repos: None,
            parallel: false,
        };

        let pr_command = PrCommand {
            title: "Test PR".to_string(),
            body: "Test body".to_string(),
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token: "test_token".to_string(),
            create_only: false,
        };

        let result = pr_command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pr_command_with_filters() {
        let repository = Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            path: Some("./test-repo".to_string()),
            branch: None,
            tags: vec!["api".to_string()],
            config_dir: None,
        };

        let config = Config {
            repositories: vec![repository],
            recipes: vec![],
        };

        let context = CommandContext {
            config,
            tag: vec!["nonexistent".to_string()],
            exclude_tag: vec![],
            repos: None,
            parallel: false,
        };

        let pr_command = PrCommand {
            title: "Test PR".to_string(),
            body: "Test body".to_string(),
            branch_name: Some("feature/test".to_string()),
            base_branch: Some("main".to_string()),
            commit_msg: Some("Test commit".to_string()),
            draft: true,
            token: "test_token".to_string(),
            create_only: true,
        };

        let result = pr_command.execute(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pr_command_execution_paths() {
        let repository = Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            path: Some("./nonexistent-path".to_string()),
            branch: None,
            tags: vec!["backend".to_string()],
            config_dir: None,
        };

        let config = Config {
            repositories: vec![repository],
            recipes: vec![],
        };

        let context = CommandContext {
            config,
            tag: vec!["backend".to_string()],
            exclude_tag: vec![],
            repos: None,
            parallel: false,
        };

        let pr_command = PrCommand {
            title: "Integration Test PR".to_string(),
            body: "Integration test body".to_string(),
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token: "test_token".to_string(),
            create_only: false,
        };

        // This will hit the error handling paths since the repo doesn't exist
        let result = pr_command.execute(&context).await;
        assert!(result.is_err()); // Expect error due to nonexistent repository
    }

    #[tokio::test]
    async fn test_pr_command_parallel_execution() {
        let repository = Repository {
            name: "test-repo-parallel".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            path: Some("./nonexistent-parallel".to_string()),
            branch: None,
            tags: vec!["test".to_string()],
            config_dir: None,
        };

        let config = Config {
            repositories: vec![repository],
            recipes: vec![],
        };

        let context = CommandContext {
            config,
            tag: vec!["test".to_string()],
            exclude_tag: vec![],
            repos: None,
            parallel: true, // Test parallel execution path
        };

        let pr_command = PrCommand {
            title: "Parallel Test PR".to_string(),
            body: "Parallel test body".to_string(),
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token: "test_token".to_string(),
            create_only: false,
        };

        // This will hit the parallel execution error handling paths
        let result = pr_command.execute(&context).await;
        assert!(result.is_err()); // Expect error due to nonexistent repository
    }

    #[tokio::test]
    async fn test_pr_command_module_exists() {
        // Test to ensure the PR command module is properly accessible
        let pr_command = PrCommand {
            title: "Module Test".to_string(),
            body: "Module test body".to_string(),
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token: "test_token".to_string(),
            create_only: false,
        };

        assert_eq!(pr_command.title, "Module Test");
        assert!(!pr_command.draft);
        assert!(!pr_command.create_only);
    }
}
