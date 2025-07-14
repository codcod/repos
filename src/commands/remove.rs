//! Remove command implementation

use super::{Command, CommandContext};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use std::fs;

/// Remove command for deleting cloned repositories
pub struct RemoveCommand;

#[async_trait]
impl Command for RemoveCommand {
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
            format!("Removing {} repositories...", repositories.len()).green()
        );

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    tokio::spawn(async move {
                        let target_dir = repo.get_target_dir();
                        tokio::task::spawn_blocking(move || {
                            if std::path::Path::new(&target_dir).exists() {
                                fs::remove_dir_all(&target_dir).map_err(anyhow::Error::from)
                            } else {
                                println!("{} | Directory does not exist", repo.name.cyan().bold());
                                Ok(())
                            }
                        })
                        .await?
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
                let target_dir = repo.get_target_dir();
                if std::path::Path::new(&target_dir).exists() {
                    if let Err(e) = fs::remove_dir_all(&target_dir) {
                        eprintln!(
                            "{} | {}",
                            repo.name.cyan().bold(),
                            format!("Error: {e}").red()
                        );
                    } else {
                        println!("{} | {}", repo.name.cyan().bold(), "Removed".green());
                    }
                } else {
                    println!("{} | Directory does not exist", repo.name.cyan().bold());
                }
            }
        }

        println!("{}", "Done removing repositories".green());
        Ok(())
    }
}
