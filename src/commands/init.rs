//! Init command implementation

use super::{Command, CommandContext};
use crate::config::{Config, RepositoryBuilder};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use std::path::Path;
use walkdir::WalkDir;

/// Init command for creating config from discovered repositories
pub struct InitCommand {
    pub output: String,
    pub overwrite: bool,
}

#[async_trait]
impl Command for InitCommand {
    async fn execute(&self, _context: &CommandContext) -> Result<()> {
        if Path::new(&self.output).exists() && !self.overwrite {
            return Err(anyhow::anyhow!(
                "Output file '{}' already exists. Use --overwrite to replace it.",
                self.output
            ));
        }

        println!("{}", "Discovering Git repositories...".green());

        let mut repositories = Vec::new();
        let current_dir = std::env::current_dir()?;

        for entry in WalkDir::new(&current_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == ".git" && entry.file_type().is_dir() {
                if let Some(repo_dir) = entry.path().parent() {
                    if let Some(name) = repo_dir.file_name().and_then(|n| n.to_str()) {
                        // Try to get remote URL
                        if let Ok(url) = get_git_remote_url(repo_dir) {
                            let repo = RepositoryBuilder::new(name.to_string(), url)
                                .with_path(
                                    repo_dir
                                        .strip_prefix(&current_dir)
                                        .unwrap_or(repo_dir)
                                        .to_string_lossy()
                                        .to_string(),
                                )
                                .build();
                            repositories.push(repo);
                        }
                    }
                }
            }
        }

        if repositories.is_empty() {
            println!(
                "{}",
                "No Git repositories found in current directory".yellow()
            );
            return Ok(());
        }

        println!(
            "{}",
            format!("Found {} repositories", repositories.len()).green()
        );

        let config = Config { repositories };
        config.save(&self.output)?;

        println!(
            "{}",
            format!("Configuration saved to '{}'", self.output).green()
        );

        Ok(())
    }
}

fn get_git_remote_url(repo_path: &Path) -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()?;

    if output.status.success() {
        let url = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(url)
    } else {
        Err(anyhow::anyhow!("Failed to get remote URL"))
    }
}
