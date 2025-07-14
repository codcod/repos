//! Clone command implementation

use super::{Command, CommandContext};
use crate::git;
use anyhow::Result;
use async_trait::async_trait;
use colored::*;

/// Clone command for cloning repositories
pub struct CloneCommand;

#[async_trait]
impl Command for CloneCommand {
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
            format!("Cloning {} repositories...", repositories.len()).green()
        );

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    tokio::spawn(async move {
                        tokio::task::spawn_blocking(move || git::clone_repository(&repo)).await?
                    })
                })
                .collect();

            for task in tasks {
                if let Err(e) = task.await? {
                    eprintln!("{}", format!("Error: {e}").red());
                }
            }
        } else {
            for repo in repositories {
                if let Err(e) = tokio::task::spawn_blocking({
                    let repo = repo.clone();
                    move || git::clone_repository(&repo)
                })
                .await?
                {
                    eprintln!("{}", format!("Error: {e}").red());
                }
            }
        }

        println!("{}", "Done cloning repositories".green());
        Ok(())
    }
}
