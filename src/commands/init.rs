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
            if entry.file_name() == ".git"
                && entry.file_type().is_dir()
                && let Some(repo_dir) = entry.path().parent()
                && let Some(name) = repo_dir.file_name().and_then(|n| n.to_str())
            {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_command_no_repositories_found() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory (empty, no git repos)
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let output_path = temp_dir.path().join("empty-config.yaml");
        let command = InitCommand {
            output: output_path.to_string_lossy().to_string(),
            overwrite: false,
        };

        let context = CommandContext {
            config: Config {
                repositories: vec![],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        let result = command.execute(&context).await;
        assert!(result.is_ok()); // Should succeed but not create file

        // Verify no config file was created
        assert!(!output_path.exists());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_no_overwrite_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("existing-config.yaml");

        // Create existing file
        fs::write(&output_path, "existing content").unwrap();

        let command = InitCommand {
            output: output_path.to_string_lossy().to_string(),
            overwrite: false, // Should not overwrite
        };

        let context = CommandContext {
            config: Config {
                repositories: vec![],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        let result = command.execute(&context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        // Verify file was not modified
        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "existing content");
    }

    #[tokio::test]
    async fn test_init_command_structure() {
        // Test that we can create the command and it has the right fields
        let command = InitCommand {
            output: "test.yaml".to_string(),
            overwrite: true,
        };

        assert_eq!(command.output, "test.yaml");
        assert!(command.overwrite);
    }
}
