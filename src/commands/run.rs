//! Run command implementation

use super::{Command, CommandContext};
use crate::runner::CommandRunner;
use crate::utils::sanitizers::{sanitize_for_filename, sanitize_script_name};
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
#[derive(Debug)]
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
            let command_suffix = sanitize_for_filename(command);
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
            let recipe_suffix = sanitize_for_filename(recipe_name);
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

                        // Ensure script path is executable from current directory
                        let executable_script_path = if relative_script_path.contains('/') {
                            relative_script_path.to_string()
                        } else {
                            format!("./{}", relative_script_path)
                        };

                        let runner = CommandRunner::new();
                        let result = if let Some(ref run_root) = run_root {
                            runner
                                .run_command_with_recipe_context(
                                    &repo,
                                    &executable_script_path,
                                    Some(run_root.to_string_lossy().as_ref()),
                                    &recipe_name,
                                    &recipe_steps,
                                )
                                .await
                        } else {
                            runner
                                .run_command_with_capture_no_logs(
                                    &repo,
                                    &executable_script_path,
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

                // Ensure script path is executable from current directory
                let executable_script_path = if relative_script_path.contains('/') {
                    relative_script_path.to_string()
                } else {
                    format!("./{}", relative_script_path)
                };

                let result = if let Some(ref run_root) = run_root {
                    runner
                        .run_command_with_recipe_context(
                            &repo,
                            &executable_script_path,
                            Some(run_root.to_string_lossy().as_ref()),
                            &recipe.name,
                            &recipe.steps,
                        )
                        .await
                } else {
                    runner
                        .run_command_with_capture_no_logs(&repo, &executable_script_path, None)
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

        // Create script directly in the repository root
        let script_label = sanitize_script_name(recipe_name);
        let script_path = repo_path.join(format!("{}.script", script_label));

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_command_for_filename() {
        assert_eq!(sanitize_for_filename("echo hello"), "echo_hello");
        assert_eq!(sanitize_for_filename("ls -la"), "ls_-la");
        assert_eq!(sanitize_for_filename("cat file.txt"), "cat_file.txt");
        assert_eq!(
            sanitize_for_filename("cmd/with/slashes"),
            "cmd_with_slashes"
        );
        assert_eq!(sanitize_for_filename("cmd:with:colons"), "cmd_with_colons");
        assert_eq!(
            sanitize_for_filename("cmd?with?special*chars"),
            "cmd_with_special_chars"
        );

        // Test length limiting
        let long_command = "a".repeat(60);
        let sanitized = sanitize_for_filename(&long_command);
        assert_eq!(sanitized.len(), 50);
        assert_eq!(sanitized, "a".repeat(50));
    }

    #[test]
    fn test_sanitize_script_name() {
        assert_eq!(sanitize_script_name("TestScript"), "testscript");
        assert_eq!(sanitize_script_name("my-script"), "my-script");
        assert_eq!(sanitize_script_name("script_name"), "script_name");
        assert_eq!(
            sanitize_script_name("script@example.com"),
            "script_example_com"
        );
        assert_eq!(sanitize_script_name("UPPERCASE"), "uppercase");
        assert_eq!(
            sanitize_script_name("script with spaces"),
            "script_with_spaces"
        );
        assert_eq!(sanitize_script_name("123-script"), "123-script");
    }

    #[test]
    fn test_run_command_constructors() {
        // Test new_command constructor
        let cmd =
            RunCommand::new_command("echo test".to_string(), false, Some(PathBuf::from("/tmp")));
        match cmd.run_type {
            RunType::Command(ref command) => assert_eq!(command, "echo test"),
            _ => panic!("Expected Command type"),
        }
        assert!(!cmd.no_save);
        assert_eq!(cmd.output_dir, Some(PathBuf::from("/tmp")));

        // Test new_recipe constructor
        let cmd = RunCommand::new_recipe("test-recipe".to_string(), true, None);
        match cmd.run_type {
            RunType::Recipe(ref recipe) => assert_eq!(recipe, "test-recipe"),
            _ => panic!("Expected Recipe type"),
        }
        assert!(cmd.no_save);
        assert_eq!(cmd.output_dir, None);

        // Test new_for_test constructor
        let cmd = RunCommand::new_for_test("test command".to_string(), "/test/output".to_string());
        match cmd.run_type {
            RunType::Command(ref command) => assert_eq!(command, "test command"),
            _ => panic!("Expected Command type"),
        }
        assert!(!cmd.no_save);
        assert_eq!(cmd.output_dir, Some(PathBuf::from("/test/output")));
    }

    #[test]
    fn test_sanitize_command_edge_cases() {
        // Test empty string
        assert_eq!(sanitize_for_filename(""), "");

        // Test string with only special characters
        assert_eq!(sanitize_for_filename("!@#$%^&*()"), "__________");

        // Test string with mixed valid and invalid characters
        assert_eq!(
            sanitize_for_filename("test-123_abc.txt!@#"),
            "test-123_abc.txt___"
        );

        // Test string exactly at limit (50 chars)
        let exactly_fifty = "a".repeat(50);
        let sanitized = sanitize_for_filename(&exactly_fifty);
        assert_eq!(sanitized.len(), 50);
        assert_eq!(sanitized, exactly_fifty);

        // Test Unicode characters (alphanumeric Unicode chars are preserved)
        assert_eq!(sanitize_for_filename("café"), "café");
        assert_eq!(sanitize_for_filename("测试-test"), "测试-test");
    }

    #[test]
    fn test_sanitize_script_name_edge_cases() {
        // Test empty string
        assert_eq!(sanitize_script_name(""), "");

        // Test string with only special characters
        assert_eq!(sanitize_script_name("!@#$%^&*()"), "__________");

        // Test string with numbers only
        assert_eq!(sanitize_script_name("12345"), "12345");

        // Test string with mixed case and special chars
        assert_eq!(
            sanitize_script_name("Test-Script_2023!"),
            "test-script_2023_"
        );

        // Test consecutive special characters
        assert_eq!(sanitize_script_name("test!!!script"), "test___script");

        // Test Unicode characters get converted (only ASCII alphanumeric preserved)
        assert_eq!(sanitize_script_name("café-script"), "caf_-script");
    }

    #[test]
    fn test_run_type_debug() {
        // Test Debug implementation for RunType enum
        let cmd_type = RunType::Command("echo test".to_string());
        let debug_str = format!("{:?}", cmd_type);
        assert!(debug_str.contains("Command"));
        assert!(debug_str.contains("echo test"));

        let recipe_type = RunType::Recipe("test-recipe".to_string());
        let debug_str = format!("{:?}", recipe_type);
        assert!(debug_str.contains("Recipe"));
        assert!(debug_str.contains("test-recipe"));
    }

    #[test]
    fn test_run_command_debug() {
        // Test Debug implementation for RunCommand struct
        let cmd = RunCommand::new_command("echo test".to_string(), true, None);
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("RunCommand"));
        assert!(debug_str.contains("no_save: true"));
        assert!(debug_str.contains("output_dir: None"));
    }
}
