//! Pull request command implementation

use super::{Command, CommandContext};
use crate::github::{self, PrOptions};
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
        let repositories = context
            .config
            .filter_repositories(context.tag.as_deref(), context.repos.as_deref());

        if repositories.is_empty() {
            let filter_desc = match (&context.tag, &context.repos) {
                (Some(tag), Some(repos)) => format!("tag '{tag}' and repositories {repos:?}"),
                (Some(tag), None) => format!("tag '{tag}'"),
                (None, Some(repos)) => format!("repositories {repos:?}"),
                (None, None) => "no repositories found".to_string(),
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
                            github::create_pr_from_workspace(&repo, &pr_options).await,
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
                match github::create_pr_from_workspace(&repo, &pr_options).await {
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
