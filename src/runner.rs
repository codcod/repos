//! Command execution runner for managing operations across multiple repositories

use crate::config::Repository;
use crate::git::Logger;
use anyhow::Result;
use serde_json;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
struct RecipeContext {
    name: String,
    steps: Vec<String>,
}

#[derive(Default)]
pub struct CommandRunner {
    logger: Logger,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a human-readable description of an exit code
    fn get_exit_code_description(exit_code: i32) -> &'static str {
        match exit_code {
            0 => "success",
            1 => "general error",
            2 => "misuse of shell builtins",
            126 => "command invoked cannot execute",
            127 => "command not found",
            128 => "invalid argument to exit",
            130 => "script terminated by Control-C",
            _ if exit_code > 128 => "terminated by signal",
            _ => "error",
        }
    }

    /// Run command and capture output for the new logging system
    pub async fn run_command_with_capture(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        self.run_command_with_capture_internal(repo, command, log_dir, false, None)
            .await
    }

    /// Run command with recipe context and capture output for the new logging system
    pub async fn run_command_with_recipe_context(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
        recipe_name: &str,
        recipe_steps: &[String],
    ) -> Result<(String, String, i32)> {
        let recipe_context = Some(RecipeContext {
            name: recipe_name.to_string(),
            steps: recipe_steps.to_vec(),
        });
        self.run_command_with_capture_internal(repo, command, log_dir, false, recipe_context)
            .await
    }

    /// Run command and capture output without creating log files (for persist mode)
    pub async fn run_command_with_capture_no_logs(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        self.run_command_with_capture_internal(repo, command, log_dir, true, None)
            .await
    }

    /// Internal implementation that allows skipping log file creation
    async fn run_command_with_capture_internal(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
        skip_log_file: bool,
        recipe_context: Option<RecipeContext>,
    ) -> Result<(String, String, i32)> {
        let repo_dir = repo.get_target_dir();

        // Check if directory exists
        if !Path::new(&repo_dir).exists() {
            anyhow::bail!("Repository directory does not exist: {}", repo_dir);
        }

        self.logger.info(repo, &format!("Running '{command}'"));

        // Execute command
        let mut cmd = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&repo_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = cmd.stdout.take().unwrap();
        let stderr = cmd.stderr.take().unwrap();

        // Handle stdout
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut content = String::new();
            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    content.push_str(&line);
                    content.push('\n');
                }
            }
            content
        });

        // Handle stderr
        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut content = String::new();

            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    content.push_str(&line);
                    content.push('\n');
                }
            }
            content
        });

        // Wait for output processing to complete and capture content
        let (stdout_result, stderr_result) = tokio::join!(stdout_handle, stderr_handle);
        let stdout_content = stdout_result.unwrap_or_default();
        let stderr_content = stderr_result.unwrap_or_default();

        // Wait for command to complete
        let status = cmd.wait()?;
        let exit_code = status.code().unwrap_or(-1);

        // Save output to files if log directory is provided and not skipping log files
        if let Some(log_dir) = log_dir
            && !skip_log_file
        {
            // Create repo-specific subdirectory
            let repo_log_dir = Path::new(log_dir).join(&repo.name);
            std::fs::create_dir_all(&repo_log_dir)?;

            // Always write metadata file with command and exit code in JSON format
            let exit_code_description = Self::get_exit_code_description(exit_code);
            let metadata_content = if let Some(ref recipe_ctx) = recipe_context {
                serde_json::json!({
                    "recipe": recipe_ctx.name,
                    "exit_code": exit_code,
                    "exit_code_description": exit_code_description,
                    "repository": repo.name,
                    "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    "recipe_steps": recipe_ctx.steps
                })
            } else {
                serde_json::json!({
                    "command": command,
                    "exit_code": exit_code,
                    "exit_code_description": exit_code_description,
                    "repository": repo.name,
                    "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                })
            };
            let metadata_file = repo_log_dir.join("metadata.json");
            std::fs::write(
                &metadata_file,
                serde_json::to_string_pretty(&metadata_content)?,
            )?;

            // Write stdout to file (even if empty, to show it was captured)
            let stdout_file = repo_log_dir.join("stdout.log");
            std::fs::write(&stdout_file, &stdout_content)?;

            // Write stderr to file (even if empty, to show it was captured)
            let stderr_file = repo_log_dir.join("stderr.log");
            std::fs::write(&stderr_file, &stderr_content)?;
        }

        // Log completion with exit code and description
        let exit_code_description = Self::get_exit_code_description(exit_code);
        if let Some(ref recipe_ctx) = recipe_context {
            self.logger.info(
                repo,
                &format!(
                    "Recipe '{}' ended with exit code {} ({})",
                    recipe_ctx.name, exit_code, exit_code_description
                ),
            );
        } else {
            self.logger.info(
                repo,
                &format!(
                    "Command '{}' ended with exit code {} ({})",
                    command, exit_code, exit_code_description
                ),
            );
        }

        // Always return the captured output, regardless of exit code
        // This allows the caller to decide how to handle failures and still log the output
        Ok((stdout_content, stderr_content, exit_code))
    }

    pub async fn run_command(
        &self,
        repo: &Repository,
        command: &str,
        _log_dir: Option<&str>,
    ) -> Result<()> {
        let repo_dir = repo.get_target_dir();

        // Check if directory exists
        if !Path::new(&repo_dir).exists() {
            anyhow::bail!("Repository directory does not exist: {}", repo_dir);
        }

        self.logger.info(repo, &format!("Running '{command}'"));

        // Execute command
        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&repo_dir)
            .status()?;

        let exit_code = status.code().unwrap_or(-1);
        let exit_code_description = Self::get_exit_code_description(exit_code);

        self.logger.info(
            repo,
            &format!(
                "Command '{}' ended with exit code {} ({})",
                command, exit_code, exit_code_description
            ),
        );

        if !status.success() {
            anyhow::bail!("Command failed with exit code: {}", exit_code);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Repository;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper function to create a test repository with git initialized.
    /// Returns the Repository object and the temporary directory.
    fn create_test_repo_with_git(name: &str, url: &str) -> (Repository, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo_path = temp_dir.path().join(name);
        fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

        // Initialize git repository
        let git_init_status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to execute git init");
        assert!(git_init_status.success(), "git init failed");

        // Configure git user for the test repo
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to configure git user name");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to configure git user email");

        let mut repo = Repository::new(name.to_string(), url.to_string());
        repo.set_config_dir(Some(temp_dir.path().to_path_buf()));
        // Override the repo path to point to our specific test subdirectory
        repo.path = Some(repo_path.to_string_lossy().to_string());

        (repo, temp_dir)
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let _runner = CommandRunner::new();
        // Verifies that the CommandRunner can be created without panicking.
    }

    #[tokio::test]
    async fn test_run_command_success() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-success", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let result = runner.run_command(&repo, "echo 'Hello World'", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_failure_with_exit_code() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-failure", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let result = runner.run_command(&repo, "exit 42", None).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Command failed with exit code: 42"));
    }

    #[tokio::test]
    async fn test_run_command_nonexistent_command() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-nonexistent", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let result = runner
            .run_command(&repo, "nonexistent_command_12345", None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_command_empty_command() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-empty", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        // An empty command should succeed (it's a no-op for the shell).
        let result = runner.run_command(&repo, "", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_repository_does_not_exist() {
        let mut repo = Repository::new(
            "nonexistent-repo".to_string(),
            "git@github.com:owner/test.git".to_string(),
        );
        // Point to a path that definitely won't exist.
        repo.path = Some("/path/that/does/not/exist/12345".to_string());

        let runner = CommandRunner::new();
        let result = runner.run_command(&repo, "echo 'test'", None).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Repository directory does not exist"));
    }

    #[tokio::test]
    async fn test_run_command_working_directory() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-wd", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let repo_path = Path::new(repo.path.as_ref().unwrap());
        let test_file = repo_path.join("testfile.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");

        // This command should succeed because it's run in the repository's directory.
        let result = runner.run_command(&repo, "ls testfile.txt", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_with_pipes() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-pipe", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let result = runner
            .run_command(&repo, "echo 'hello world' | grep 'world'", None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_with_log_directory() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-log", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(&repo, "echo 'Logged output'", Some(&log_dir_str))
            .await;
        assert!(result.is_ok());

        // Verify log files are created in repo-specific subdirectory
        let repo_log_dir = log_dir.join(&repo.name);
        assert!(repo_log_dir.exists(), "Repo log directory should exist");

        let stdout_file = repo_log_dir.join("stdout.log");
        let metadata_file = repo_log_dir.join("metadata.json");

        assert!(stdout_file.exists(), "stdout.log should exist");
        assert!(metadata_file.exists(), "metadata.json should exist");

        let stdout_content = std::fs::read_to_string(&stdout_file).unwrap();
        assert!(stdout_content.contains("Logged output"));

        let metadata_content = std::fs::read_to_string(&metadata_file).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content).unwrap();
        assert_eq!(metadata["command"], "echo 'Logged output'");
        assert_eq!(metadata["exit_code"], 0);
        assert_eq!(metadata["exit_code_description"], "success");
    }

    #[tokio::test]
    async fn test_run_command_log_file_content_and_headers() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-log-content", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(
                &repo,
                "echo 'stdout message'; echo 'stderr message' >&2",
                Some(&log_dir_str),
            )
            .await;
        assert!(result.is_ok());

        // Verify log files are created with proper content
        let repo_log_dir = log_dir.join(&repo.name);
        assert!(repo_log_dir.exists(), "Repo log directory should exist");

        let stdout_file = repo_log_dir.join("stdout.log");
        let stderr_file = repo_log_dir.join("stderr.log");
        let metadata_file = repo_log_dir.join("metadata.json");

        assert!(stdout_file.exists(), "stdout.log should exist");
        assert!(stderr_file.exists(), "stderr.log should exist");
        assert!(metadata_file.exists(), "metadata.json should exist");

        let stdout_content = std::fs::read_to_string(&stdout_file).unwrap();
        assert!(stdout_content.contains("stdout message"));

        let stderr_content = std::fs::read_to_string(&stderr_file).unwrap();
        assert!(stderr_content.contains("stderr message"));

        let metadata_content = std::fs::read_to_string(&metadata_file).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content).unwrap();
        assert_eq!(metadata["exit_code"], 0);
        assert_eq!(metadata["exit_code_description"], "success");
    }

    #[tokio::test]
    async fn test_run_command_log_file_creation_error() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-log-error", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        // Create a file where the log directory should be, causing a creation error.
        let invalid_log_dir = temp_dir.path().join("invalid_log_dir");
        fs::write(&invalid_log_dir, "I am a file").unwrap();

        let result = runner
            .run_command(
                &repo,
                "echo 'test'",
                Some(&invalid_log_dir.to_string_lossy()),
            )
            .await;
        // Should succeed now since we don't create log files
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_very_long_output() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-long-output", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();
        let log_dir = temp_dir.path().join("logs");

        let result = runner
            .run_command(
                &repo,
                "for i in $(seq 1 100); do echo \"Line $i\"; done",
                Some(&log_dir.to_string_lossy()),
            )
            .await;
        assert!(result.is_ok());

        // Verify no log files are created
        let log_files: Vec<_> = if log_dir.exists() {
            fs::read_dir(&log_dir)
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        } else {
            Vec::new()
        };
        assert!(log_files.is_empty(), "No log files should be created");
    }

    #[tokio::test]
    async fn test_run_command_special_characters_in_repo_name() {
        let (repo, temp_dir) = create_test_repo_with_git(
            "test-repo_with-special.chars",
            "https://github.com/test/repo",
        );
        let runner = CommandRunner::new();
        let log_dir = temp_dir.path().join("logs");

        let result = runner
            .run_command(
                &repo,
                "echo 'test with special chars'",
                Some(&log_dir.to_string_lossy()),
            )
            .await;
        assert!(result.is_ok());

        // Verify no log files are created
        let log_files: Vec<_> = if log_dir.exists() {
            fs::read_dir(&log_dir)
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        } else {
            Vec::new()
        };
        assert!(log_files.is_empty(), "No log files should be created");
    }

    #[tokio::test]
    async fn test_run_command_with_capture_success() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-capture", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(&repo, "echo 'captured output'", Some(&log_dir_str))
            .await;

        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.contains("captured output"));
        assert!(stderr.is_empty());
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_stderr() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-capture-stderr", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(&repo, "echo 'error message' >&2", Some(&log_dir_str))
            .await;

        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.is_empty());
        assert!(stderr.contains("error message"));
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_mixed_output() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-capture-mixed", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(
                &repo,
                "echo 'stdout message' && echo 'stderr message' >&2",
                Some(&log_dir_str),
            )
            .await;

        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.contains("stdout message"));
        assert!(stderr.contains("stderr message"));
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_failure() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-capture-fail", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(&repo, "exit 1", Some(&log_dir_str))
            .await;

        // Should return Ok with exit code 1 (failure is indicated by exit code, not error)
        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
        assert_eq!(exit_code, 1);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_no_log_dir() {
        let (repo, _temp_dir) =
            create_test_repo_with_git("test-capture-no-log", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let result = runner
            .run_command_with_capture(&repo, "echo 'no log dir'", None)
            .await;

        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.contains("no log dir"));
        assert!(stderr.is_empty());
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_long_output() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-capture-long", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command_with_capture(
                &repo,
                "for i in $(seq 1 50); do echo \"Line $i\"; done",
                Some(&log_dir_str),
            )
            .await;

        assert!(result.is_ok());
        let (stdout, stderr, exit_code) = result.unwrap();
        assert!(stdout.contains("Line 1"));
        assert!(stdout.contains("Line 25"));
        assert!(stdout.contains("Line 50"));
        assert!(stderr.is_empty());
        assert_eq!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_run_command_with_capture_nonexistent_directory() {
        let repo = Repository {
            name: "nonexistent-repo".to_string(),
            url: "https://github.com/test/nonexistent".to_string(),
            tags: vec!["test".to_string()],
            path: Some("/nonexistent/path".to_string()),
            branch: None,
            config_dir: None,
        };
        let runner = CommandRunner::new();

        let result = runner
            .run_command_with_capture(&repo, "echo 'test'", None)
            .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Repository directory does not exist")
        );
    }
}
