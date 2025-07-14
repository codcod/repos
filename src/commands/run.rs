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

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let runner = &runner;
                    let command = self.command.clone();
                    let log_dir = self.log_dir.clone();
                    async move { runner.run_command(&repo, &command, Some(&log_dir)).await }
                })
                .collect();

            for task in tasks {
                if let Err(e) = task.await {
                    eprintln!("{}", format!("Error: {e}").red());
                }
            }
        } else {
            for repo in repositories {
                if let Err(e) = runner
                    .run_command(&repo, &self.command, Some(&self.log_dir))
                    .await
                {
                    eprintln!(
                        "{} | {}",
                        repo.name.cyan().bold(),
                        format!("Error: {e}").red()
                    );
                }
            }
        }

        println!("{}", "Done running commands".green());
        Ok(())
    }
}
