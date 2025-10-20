//! Command execution runner for managing operations across multiple repositories

use crate::config::Repository;
use crate::git::Logger;
use anyhow::Result;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Default)]
pub struct CommandRunner {
    logger: Logger,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run command and capture output for the new logging system
    pub async fn run_command_with_capture(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        self.run_command_with_capture_internal(repo, command, log_dir, false)
            .await
    }

    /// Run command and capture output without creating log files (for persist mode)
    pub async fn run_command_with_capture_no_logs(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<(String, String, i32)> {
        self.run_command_with_capture_internal(repo, command, log_dir, true)
            .await
    }

    /// Internal implementation that allows skipping log file creation
    async fn run_command_with_capture_internal(
        &self,
        repo: &Repository,
        command: &str,
        _log_dir: Option<&str>,
        _skip_log_file: bool,
    ) -> Result<(String, String, i32)> {
        let repo_dir = repo.get_target_dir();

        // Check if directory exists
        if !Path::new(&repo_dir).exists() {
            anyhow::bail!("Repository directory does not exist: {}", repo_dir);
        }

        // No longer create log files - all output handled by persist system
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

        // No longer create log files - all output is handled by the new persist system
        self.logger.info(repo, &format!("Running '{command}'"));

        // Execute command
        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&repo_dir)
            .status()?;

        if !status.success() {
            anyhow::bail!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
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
            .run_command(&repo, "echo 'Logged output'", Some(&log_dir_str))
            .await;
        assert!(result.is_ok());

        // No log files are created anymore - the persist system handles output capture
        // The log_dir parameter is no longer used for file creation
        let log_files: Vec<_> = if log_dir.exists() {
            fs::read_dir(&log_dir)
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        } else {
            Vec::new()
        };
        assert!(
            log_files.is_empty(),
            "No log files should be created - use persist system instead"
        );
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

        // Verify no log files are created (we now use persist system instead)
        let log_files: Vec<_> = if log_dir.exists() {
            fs::read_dir(&log_dir)
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        } else {
            Vec::new()
        };
        assert!(
            log_files.is_empty(),
            "No log files should be created with new persist-only system"
        );
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
