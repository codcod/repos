//! Run command implementation

use super::{Command, CommandContext};
use crate::runner::CommandRunner;
use anyhow::Result;
use async_trait::async_trait;

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

/// Run command for executing commands in repositories
pub struct RunCommand {
    pub command: String,
    pub no_save: bool,
    pub output_dir: Option<PathBuf>,
}

#[async_trait]
impl Command for RunCommand {
    async fn execute(&self, context: &CommandContext) -> Result<()> {
        let repositories = context.config.filter_repositories(
            &context.tag,
            &context.exclude_tag,
            context.repos.as_deref(),
        );

        if repositories.is_empty() {
            return Ok(());
        }

        let runner = CommandRunner::new();

        // Setup persistent output directory if saving is enabled
        let run_root = if !self.no_save {
            // Use local time instead of UTC
            let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            // Sanitize command for directory name
            let command_suffix = Self::sanitize_command_for_filename(&self.command);
            // Use provided output directory or default to "output"
            let base_dir = self
                .output_dir
                .as_ref()
                .unwrap_or(&PathBuf::from("output"))
                .join("runs");
            let run_dir = base_dir.join(format!("{}_{}", timestamp, command_suffix));
            create_dir_all(&run_dir)?;
            Some(run_dir)
        } else {
            None
        };

        let mut errors = Vec::new();
        let mut successful = 0;

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let runner = &runner;
                    let command = self.command.clone();
                    let no_save = self.no_save;
                    async move {
                        let result = if !no_save {
                            runner
                                .run_command_with_capture_no_logs(&repo, &command, None)
                                .await
                        } else {
                            runner
                                .run_command(&repo, &command, None)
                                .await
                                .map(|_| (String::new(), String::new(), 0))
                        };
                        (repo, result)
                    }
                })
                .collect();

            for task in tasks {
                let (repo, result) = task.await;
                match result {
                    Ok((stdout, stderr, exit_code)) => {
                        if exit_code == 0 {
                            successful += 1;
                        } else {
                            errors.push((
                                repo.name.clone(),
                                anyhow::anyhow!("Command failed with exit code: {}", exit_code),
                            ));
                        }

                        // Save output to individual files
                        if let Some(ref run_dir) = run_root {
                            self.save_repo_output(&repo, &stdout, &stderr, run_dir)?;
                        }
                    }
                    Err(e) => {
                        errors.push((repo.name.clone(), e));
                    }
                }
            }
        } else {
            for repo in repositories {
                let result = if !self.no_save {
                    runner
                        .run_command_with_capture_no_logs(&repo, &self.command, None)
                        .await
                } else {
                    runner
                        .run_command(&repo, &self.command, None)
                        .await
                        .map(|_| (String::new(), String::new(), 0))
                };

                match result {
                    Ok((stdout, stderr, exit_code)) => {
                        if exit_code == 0 {
                            successful += 1;
                        } else {
                            errors.push((
                                repo.name.clone(),
                                anyhow::anyhow!("Command failed with exit code: {}", exit_code),
                            ));
                        }

                        // Save output to individual files
                        if let Some(ref run_dir) = run_root {
                            self.save_repo_output(&repo, &stdout, &stderr, run_dir)?;
                        }
                    }
                    Err(e) => {
                        errors.push((repo.name.clone(), e));
                    }
                }
            }
        }

        // Check if all operations failed
        if !errors.is_empty() && successful == 0 {
            return Err(anyhow::anyhow!(
                "All command executions failed. First error: {}",
                errors[0].1
            ));
        }

        Ok(())
    }
}

impl RunCommand {
    /// Create a new RunCommand with default settings for testing
    pub fn new_for_test(command: String, output_dir: String) -> Self {
        Self {
            command,
            no_save: false,
            output_dir: Some(PathBuf::from(output_dir)),
        }
    }

    /// Sanitize command string for use in directory names
    fn sanitize_command_for_filename(command: &str) -> String {
        command
            .chars()
            .map(|c| match c {
                ' ' => '_',
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => c,
                _ => '_',
            })
            .collect::<String>()
            .chars()
            .take(50) // Limit length to avoid overly long directory names
            .collect()
    }

    /// Save individual repository output to separate files
    fn save_repo_output(
        &self,
        repo: &crate::config::Repository,
        stdout: &str,
        stderr: &str,
        run_dir: &Path,
    ) -> Result<()> {
        let safe_name = repo.name.replace(['/', '\\', ':'], "_");

        // Save stdout
        if !stdout.is_empty() {
            let stdout_path = run_dir.join(format!("{}.stdout", safe_name));
            std::fs::write(stdout_path, stdout)?;
        }

        // Save stderr
        if !stderr.is_empty() {
            let stderr_path = run_dir.join(format!("{}.stderr", safe_name));
            std::fs::write(stderr_path, stderr)?;
        }

        Ok(())
    }
}
