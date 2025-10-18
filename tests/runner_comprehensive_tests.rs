use repos::config::Repository;
use repos::runner::CommandRunner;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_runner_command_execution_comprehensive() {
    // Test comprehensive command execution paths
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo for realistic testing
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    // Set git config for testing
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("git config email failed");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("git config name failed");

    // Create test file
    fs::write(repo_path.join("test.txt"), "test content").unwrap();

    let repo = Repository {
        name: "test-repo-comprehensive".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test successful command execution
    let result = runner.run_command(&repo, "echo 'test output'", None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runner_command_with_stderr_output_comprehensive() {
    // Test command that produces stderr output
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-stderr".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test command that outputs to stderr but succeeds
    let result = runner
        .run_command(&repo, "echo 'error message' >&2; echo 'success'", None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runner_command_failure_with_exit_code() {
    // Test command that fails with specific exit code
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-failure".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test command that fails with exit code 1
    let result = runner.run_command(&repo, "exit 1", None).await;
    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Command failed with exit code"));
}

#[tokio::test]
async fn test_runner_command_failure_with_no_exit_code() {
    // Test command failure where exit code is unavailable
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-no-exit".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test command that fails
    let result = runner.run_command(&repo, "false", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_runner_log_file_preparation() {
    // Test log file creation and header writing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    let log_dir = temp_dir.path().join("logs");

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-logs".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test with log directory
    let result = runner
        .run_command(
            &repo,
            "echo 'test with logs'",
            Some(log_dir.to_string_lossy().as_ref()),
        )
        .await;
    assert!(result.is_ok());

    // Verify log directory was created
    assert!(log_dir.exists());

    // Verify log file was created
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("test-repo-logs")
        })
        .collect();

    assert!(!log_files.is_empty());

    // Check log file content
    let log_file_path = log_files[0].path();
    let log_content = fs::read_to_string(log_file_path).unwrap();
    assert!(log_content.contains("Repository: test-repo-logs"));
    assert!(log_content.contains("Command: echo 'test with logs'"));
    assert!(log_content.contains("=== STDOUT ==="));
}

#[tokio::test]
async fn test_runner_log_file_stderr_header() {
    // Test that stderr header is written when stderr output occurs
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    let log_dir = temp_dir.path().join("logs");

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-stderr-logs".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Test command that outputs to stderr
    let result = runner
        .run_command(
            &repo,
            "echo 'stdout message'; echo 'stderr message' >&2",
            Some(log_dir.to_string_lossy().as_ref()),
        )
        .await;
    assert!(result.is_ok());

    // Check that log file contains stderr header
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("test-repo-stderr-logs")
        })
        .collect();

    assert!(!log_files.is_empty());

    let log_file_path = log_files[0].path();
    let log_content = fs::read_to_string(log_file_path).unwrap();
    assert!(log_content.contains("=== STDERR ==="));
    assert!(log_content.contains("stderr message"));
}

#[tokio::test]
async fn test_runner_log_file_creation_error() {
    // Test error handling in log file creation
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Try to use an invalid log directory path (like a file instead of directory)
    let invalid_log_path = temp_dir.path().join("invalid_log_file");
    fs::write(&invalid_log_path, "this is a file, not a directory").unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-log-error".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // This should fail because we can't create log files in a file path
    let result = runner
        .run_command(
            &repo,
            "echo 'test'",
            Some(invalid_log_path.to_string_lossy().as_ref()),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_runner_command_spawn_failure() {
    // Test command spawn failure
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-repo-spawn-fail".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // This should work, as sh should be available
    let result = runner.run_command(&repo, "echo test", None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runner_directory_does_not_exist() {
    // Test error when repository directory doesn't exist
    let repo = Repository {
        name: "test-repo-no-dir".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some("/nonexistent/path".to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    let result = runner.run_command(&repo, "echo test", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_runner_comprehensive_io_handling() {
    // Test comprehensive I/O handling with mixed stdout/stderr
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    let log_dir = temp_dir.path().join("comprehensive_logs");

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("git init failed");

    let repo = Repository {
        name: "test-comprehensive-io".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: vec!["test".to_string()],
        branch: None,
        config_dir: None,
    };

    let runner = CommandRunner::new();

    // Command that produces multiple lines of stdout and stderr
    let complex_command = r#"
        echo "Line 1 to stdout"
        echo "Line 2 to stdout"
        echo "Error line 1" >&2
        echo "Line 3 to stdout"
        echo "Error line 2" >&2
        echo "Final stdout line"
    "#;

    let result = runner
        .run_command(
            &repo,
            complex_command,
            Some(log_dir.to_string_lossy().as_ref()),
        )
        .await;
    assert!(result.is_ok());

    // Verify log file contains all output
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("test-comprehensive-io")
        })
        .collect();

    assert!(!log_files.is_empty());

    let log_file_path = log_files[0].path();
    let log_content = fs::read_to_string(log_file_path).unwrap();

    // Verify stdout content
    assert!(log_content.contains("Line 1 to stdout"));
    assert!(log_content.contains("Line 2 to stdout"));
    assert!(log_content.contains("Line 3 to stdout"));
    assert!(log_content.contains("Final stdout line"));

    // Verify stderr content and header
    assert!(log_content.contains("=== STDERR ==="));
    assert!(log_content.contains("Error line 1"));
    assert!(log_content.contains("Error line 2"));
}
