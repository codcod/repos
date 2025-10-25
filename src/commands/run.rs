//! Run command implementation

use super::{Command, CommandContext};
use crate::runner::CommandRunner;
use anyhow::Result;
use async_trait::async_trait;

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum RunType {
    Command(String),
    Recipe(String),
}

/// Run command for executing commands or recipes in repositories
pub struct RunCommand {
    pub run_type: RunType,
    pub no_save: bool,
    pub output_dir: Option<PathBuf>,
}

impl RunCommand {
    pub fn new_command(command: String, no_save: bool, output_dir: Option<PathBuf>) -> Self {
        Self {
            run_type: RunType::Command(command),
            no_save,
            output_dir,
        }
    }

    pub fn new_recipe(recipe_name: String, no_save: bool, output_dir: Option<PathBuf>) -> Self {
        Self {
            run_type: RunType::Recipe(recipe_name),
            no_save,
            output_dir,
        }
    }
}

#[async_trait]
impl Command for RunCommand {
    async fn execute(&self, context: &CommandContext) -> Result<()> {
        match &self.run_type {
            RunType::Command(command) => self.execute_command(context, command).await,
            RunType::Recipe(recipe_name) => self.execute_recipe(context, recipe_name).await,
        }
    }
}

impl RunCommand {
    /// Create a new RunCommand with default settings for testing
    pub fn new_for_test(command: String, output_dir: String) -> Self {
        Self {
            run_type: RunType::Command(command),
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

    async fn execute_command(&self, context: &CommandContext, command: &str) -> Result<()> {
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
            let command_suffix = Self::sanitize_command_for_filename(command);
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

        if context.parallel {
            // Parallel execution
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let command = command.to_string();
                    let run_root = run_root.clone();
                    async move {
                        let runner = CommandRunner::new();
                        if let Some(ref run_root) = run_root {
                            runner
                                .run_command_with_capture(
                                    &repo,
                                    &command,
                                    Some(run_root.to_string_lossy().as_ref()),
                                )
                                .await
                        } else {
                            runner
                                .run_command_with_capture_no_logs(&repo, &command, None)
                                .await
                        }
                    }
                })
                .collect();

            futures::future::join_all(tasks).await;
        } else {
            // Sequential execution
            for repo in repositories {
                if let Some(ref run_root) = run_root {
                    runner
                        .run_command_with_capture(
                            &repo,
                            command,
                            Some(run_root.to_string_lossy().as_ref()),
                        )
                        .await?;
                } else {
                    runner.run_command(&repo, command, None).await?;
                }
            }
        }

        Ok(())
    }

    async fn execute_recipe(&self, context: &CommandContext, recipe_name: &str) -> Result<()> {
        // Find the recipe
        let recipe = context
            .config
            .find_recipe(recipe_name)
            .ok_or_else(|| anyhow::anyhow!("Recipe '{}' not found", recipe_name))?;

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
            // Sanitize recipe name for directory name
            let recipe_suffix = Self::sanitize_command_for_filename(recipe_name);
            // Use provided output directory or default to "output"
            let base_dir = self
                .output_dir
                .as_ref()
                .unwrap_or(&PathBuf::from("output"))
                .join("runs");
            let run_dir = base_dir.join(format!("{}_{}", timestamp, recipe_suffix));
            create_dir_all(&run_dir)?;
            Some(run_dir)
        } else {
            None
        };

        if context.parallel {
            // Parallel execution
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let recipe_steps = recipe.steps.clone();
                    let recipe_name = recipe.name.clone();
                    let run_root = run_root.clone();
                    async move {
                        let script_path =
                            Self::materialize_script(&repo, &recipe_name, &recipe_steps).await?;

                        // Convert absolute script path to relative path from repository directory
                        let repo_target_dir = repo.get_target_dir();
                        let repo_dir = Path::new(&repo_target_dir);
                        let relative_script_path = script_path
                            .strip_prefix(repo_dir)
                            .unwrap_or(&script_path)
                            .to_string_lossy();

                        let runner = CommandRunner::new();
                        let result = if let Some(ref run_root) = run_root {
                            runner
                                .run_command_with_recipe_context(
                                    &repo,
                                    &relative_script_path,
                                    Some(run_root.to_string_lossy().as_ref()),
                                    &recipe_name,
                                    &recipe_steps,
                                )
                                .await
                        } else {
                            runner
                                .run_command_with_capture_no_logs(
                                    &repo,
                                    &relative_script_path,
                                    None,
                                )
                                .await
                        };
                        // Optionally remove script file after execution
                        let _ = std::fs::remove_file(script_path);
                        result
                    }
                })
                .collect();

            futures::future::join_all(tasks).await;
        } else {
            // Sequential execution
            for repo in repositories {
                let script_path =
                    Self::materialize_script(&repo, &recipe.name, &recipe.steps).await?;

                // Convert absolute script path to relative path from repository directory
                let repo_target_dir = repo.get_target_dir();
                let repo_dir = Path::new(&repo_target_dir);
                let relative_script_path = script_path
                    .strip_prefix(repo_dir)
                    .unwrap_or(&script_path)
                    .to_string_lossy();

                let result = if let Some(ref run_root) = run_root {
                    runner
                        .run_command_with_recipe_context(
                            &repo,
                            &relative_script_path,
                            Some(run_root.to_string_lossy().as_ref()),
                            &recipe.name,
                            &recipe.steps,
                        )
                        .await
                } else {
                    runner
                        .run_command_with_capture_no_logs(&repo, &relative_script_path, None)
                        .await
                };
                // Optionally remove script file after execution
                let _ = std::fs::remove_file(script_path);
                result?;
            }
        }

        Ok(())
    }

    async fn materialize_script(
        repo: &crate::config::Repository,
        recipe_name: &str,
        steps: &[String],
    ) -> Result<PathBuf> {
        let target_dir = repo.get_target_dir();
        let repo_path = Path::new(&target_dir);
        let recipes_dir = repo_path.join(".repos").join("recipes");
        create_dir_all(&recipes_dir)?;

        let script_label = Self::sanitize_script_name(recipe_name);
        let script_path = recipes_dir.join(format!("{}.script", script_label));

        // Join all steps with newlines to create the script content
        let script_content = steps.join("\n");
        let content = if script_content.starts_with("#!") {
            script_content
        } else {
            format!("#!/bin/sh\n{}", script_content)
        };

        std::fs::write(&script_path, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = std::fs::metadata(&script_path)?.permissions();
            perm.set_mode(0o750);
            std::fs::set_permissions(&script_path, perm)?;
        }

        Ok(script_path)
    }

    fn sanitize_script_name(name: &str) -> String {
        let mut out = String::with_capacity(name.len());
        for c in name.chars() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                out.push(c.to_ascii_lowercase());
            } else {
                out.push('_');
            }
        }
        out
    }
}
