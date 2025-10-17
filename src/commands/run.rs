//! Run command implementation

use super::{Command, CommandContext};
use crate::runner::CommandRunner;
use anyhow::Result;
use async_trait::async_trait;
use colored::*;

/// Run command for executing commands in repositories
pub struct RunCommand {
    pub command: String,
    pub log_dir: String,
}

#[async_trait]
impl Command for RunCommand {
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
                "Running '{}' in {} repositories...",
                self.command,
                repositories.len()
            )
            .green()
        );

        let runner = CommandRunner::new();

        let mut errors = Vec::new();
        let mut successful = 0;

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let runner = &runner;
                    let command = self.command.clone();
                    let log_dir = self.log_dir.clone();
                    async move {
                        (
                            repo.name.clone(),
                            runner.run_command(&repo, &command, Some(&log_dir)).await,
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
                match runner
                    .run_command(&repo, &self.command, Some(&self.log_dir))
                    .await
                {
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
            println!("{}", "Done running commands".green());
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
                    "All command executions failed. First error: {}",
                    errors[0].1
                ));
            }
        }

        Ok(())
    }
}
