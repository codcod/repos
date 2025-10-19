//! Command execution runner for managing operations across multiple repositories

use crate::config::Repository;
use crate::git::Logger;
use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct CommandRunner {
    logger: Logger,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run_command(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<()> {
        let repo_dir = repo.get_target_dir();

        // Check if directory exists
        if !Path::new(&repo_dir).exists() {
            anyhow::bail!("Repository directory does not exist: {}", repo_dir);
        }

        // Prepare log file if log directory is specified
        let log_file = if let Some(log_dir) = log_dir {
            Some(self.prepare_log_file(repo, log_dir, command, &repo_dir)?)
        } else {
            None
        };

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

        let log_file = Arc::new(Mutex::new(log_file));
        let repo_name = repo.name.clone();

        // Handle stdout
        let stdout_log_file = Arc::clone(&log_file);
        let stdout_repo_name = repo_name.clone();
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            // Note: We explicitly handle Result instead of using .flatten()
            // to avoid infinite loops on repeated I/O errors
            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Print to console with colored repo name
                    println!("{} | {line}", stdout_repo_name.cyan());

                    // Write to log file if available
                    if let Some(ref mut log_file) = *stdout_log_file.lock().await {
                        writeln!(log_file, "{stdout_repo_name} | {line}").ok();
                        log_file.flush().ok();
                    }
                }
            }
        });

        // Handle stderr
        let stderr_log_file = Arc::clone(&log_file);
        let stderr_repo_name = repo_name.clone();
        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut header_written = false;

            // Note: We explicitly handle Result instead of using .flatten()
            // to avoid infinite loops on repeated I/O errors
            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Print to console with colored repo name
                    eprintln!("{} | {line}", stderr_repo_name.red().bold());

                    // Write to log file if available
                    if let Some(ref mut log_file) = *stderr_log_file.lock().await {
                        if !header_written {
                            writeln!(log_file, "\n=== STDERR ===").ok();
                            header_written = true;
                        }
                        writeln!(log_file, "{stderr_repo_name} | {line}").ok();
                        log_file.flush().ok();
                    }
                }
            }
        });

        // Wait for output processing to complete
        let _ = tokio::join!(stdout_handle, stderr_handle);

        // Wait for command to complete
        let status = cmd.wait()?;

        if !status.success() {
            anyhow::bail!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }

    fn prepare_log_file(
        &self,
        repo: &Repository,
        log_dir: &str,
        command: &str,
        repo_dir: &str,
    ) -> Result<File> {
        // Create log directory if it doesn't exist
        create_dir_all(log_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let log_file_path = format!("{}/{}_{}.log", log_dir, repo.name, timestamp);

        let mut log_file = File::create(&log_file_path)?;

        // Write header information
        writeln!(log_file, "Repository: {}", repo.name)?;
        writeln!(log_file, "Command: {command}")?;
        writeln!(log_file, "Directory: {repo_dir}")?;
        writeln!(log_file, "Timestamp: {}", Utc::now().to_rfc3339())?;
        writeln!(log_file, "\n=== STDOUT ===")?;

        Ok(log_file)
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
            .run_command(&repo, "echo 'Logged output'", Some(&log_dir_str))
            .await;
        assert!(result.is_ok());

        assert!(log_dir.exists());
        let log_files: Vec<_> = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert!(!log_files.is_empty(), "Log file should have been created");
    }

    #[tokio::test]
    async fn test_run_command_log_file_content_and_headers() {
        let (repo, temp_dir) =
            create_test_repo_with_git("test-log-content", "git@github.com:owner/test.git");
        let runner = CommandRunner::new();

        let log_dir = temp_dir.path().join("logs");
        let log_dir_str = log_dir.to_string_lossy().to_string();

        let result = runner
            .run_command(
                &repo,
                "echo 'stdout message'; echo 'stderr message' >&2",
                Some(&log_dir_str),
            )
            .await;
        assert!(result.is_ok());

        let log_file_path = fs::read_dir(&log_dir)
            .unwrap()
            .filter_map(Result::ok)
            .next()
            .expect("No log file found")
            .path();

        let log_content = fs::read_to_string(log_file_path).unwrap();

        assert!(log_content.contains("Repository: test-log-content"));
        assert!(log_content.contains("Command: echo 'stdout message'; echo 'stderr message' >&2"));
        assert!(log_content.contains("Directory:"));
        assert!(log_content.contains("Timestamp:"));
        assert!(log_content.contains("=== STDOUT ==="));
        assert!(log_content.contains("stdout message"));
        assert!(log_content.contains("=== STDERR ==="));
        assert!(log_content.contains("stderr message"));
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
        assert!(result.is_err());
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

        let log_file_path = fs::read_dir(&log_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let log_content = fs::read_to_string(log_file_path).unwrap();
        assert!(log_content.contains("Line 1"));
        assert!(log_content.contains("Line 50"));
        assert!(log_content.contains("Line 100"));
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

        let log_file_name = fs::read_dir(&log_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name();
        // The filename should be sanitized (dots become dashes, so "test-repo_with-special.chars" becomes something with dashes)
        let filename = log_file_name.to_string_lossy();
        assert!(
            filename.contains("test-repo")
                && filename.contains("special")
                && filename.contains("chars")
        );
    }
}
