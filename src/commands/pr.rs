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

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let pr_options = pr_options.clone();
                    async move { github::create_pull_request(&repo, &pr_options).await }
                })
                .collect();

            for task in tasks {
                if let Err(e) = task.await {
                    eprintln!("{}", format!("Error: {e}").red());
                }
            }
        } else {
            for repo in repositories {
                if let Err(e) = github::create_pull_request(&repo, &pr_options).await {
                    eprintln!(
                        "{} | {}",
                        repo.name.cyan().bold(),
                        format!("Error: {e}").red()
                    );
                }
            }
        }

        println!("{}", "Done processing pull requests".green());
        Ok(())
    }
}
