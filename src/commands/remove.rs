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

        let mut errors = Vec::new();
        let mut successful = 0;

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let repo_name = repo.name.clone();
                    tokio::spawn(async move {
                        let target_dir = repo.get_target_dir();
                        let result = tokio::task::spawn_blocking(move || {
                            if std::path::Path::new(&target_dir).exists() {
                                fs::remove_dir_all(&target_dir).map_err(anyhow::Error::from)
                            } else {
                                println!("{} | Directory does not exist", repo.name.cyan().bold());
                                Ok(())
                            }
                        })
                        .await?;
                        Ok::<_, anyhow::Error>((repo_name, result))
                    })
                })
                .collect();

            for task in tasks {
                match task.await? {
                    Ok((_, Ok(_))) => successful += 1,
                    Ok((repo_name, Err(e))) => {
                        eprintln!("{}", format!("Error: {e}").red());
                        errors.push((repo_name, e));
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Task error: {e}").red());
                        errors.push(("unknown".to_string(), e));
                    }
                }
            }
        } else {
            for repo in repositories {
                let target_dir = repo.get_target_dir();
                if std::path::Path::new(&target_dir).exists() {
                    match fs::remove_dir_all(&target_dir) {
                        Ok(_) => {
                            println!("{} | {}", repo.name.cyan().bold(), "Removed".green());
                            successful += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "{} | {}",
                                repo.name.cyan().bold(),
                                format!("Error: {e}").red()
                            );
                            errors.push((repo.name.clone(), e.into()));
                        }
                    }
                } else {
                    println!("{} | Directory does not exist", repo.name.cyan().bold());
                    successful += 1; // Count as success since the desired state is achieved
                }
            }
        }

        // Report summary
        if errors.is_empty() {
            println!("{}", "Done removing repositories".green());
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
                    "All removal operations failed. First error: {}",
                    errors[0].1
                ));
            }
        }

        Ok(())
    }
}
