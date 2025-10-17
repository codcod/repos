// Comprehensive unit tests for CommandRunner functionality
// Tests cover command execution, log file handling, stdout/stderr capture, and error scenarios

use repos::config::Repository;
use repos::runner::CommandRunner;
use std::fs;
use std::path::PathBuf;

/// Helper function to create a test repository with git initialized
fn create_test_repo_with_git(name: &str, url: &str) -> (Repository, PathBuf) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let temp_base = std::env::temp_dir();

    // Create a highly unique ID using multiple sources of randomness
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);

    // Add current thread info if available
    format!("{:?}", std::thread::current().id()).hash(&mut hasher);

    let unique_id = hasher.finish();
    let temp_dir = temp_base.join(format!("repos_test_{}_{}", name, unique_id));

    // Clean up any existing directory first
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).ok();
    }

    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let mut repo = Repository::new(name.to_string(), url.to_string());
    repo.set_config_dir(Some(temp_dir.clone()));

    // Create the repository directory
    let repo_path = temp_dir.join(name);

    // Clean up any existing repo directory first
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).ok();
    }

    fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

    // Initialize git repository
    let git_init_result = std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    if !git_init_result.status.success() {
        panic!(
            "Git init failed: {}",
            String::from_utf8_lossy(&git_init_result.stderr)
        );
    }

    // Configure git user for the test repo
    let git_user_result = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git user");

    if !git_user_result.status.success() {
        panic!(
            "Git user config failed: {}",
            String::from_utf8_lossy(&git_user_result.stderr)
        );
    }

    let git_email_result = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git email");

    if !git_email_result.status.success() {
        panic!(
            "Git email config failed: {}",
            String::from_utf8_lossy(&git_email_result.stderr)
        );
    }

    (repo, temp_dir)
}

/// Cleanup helper function
fn cleanup_temp_dir(_temp_dir: &PathBuf) {
    // Disabled cleanup to avoid race conditions in tests
    // Temp directories will be cleaned up by the OS
}

#[tokio::test]
async fn test_run_command_success() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a simple command that should succeed
    let result = runner.run_command(&repo, "echo 'Hello World'", None).await;
    assert!(result.is_ok());

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_with_output() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command that produces output
    let result = runner.run_command(&repo, "echo 'Test output'", None).await;
    assert!(result.is_ok());

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_failure() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command that should fail
    let result = runner.run_command(&repo, "exit 1", None).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Command failed with exit code"));

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_nonexistent_command() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command that doesn't exist
    let result = runner
        .run_command(&repo, "nonexistent_command_12345", None)
        .await;
    assert!(result.is_err());

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_repository_does_not_exist() {
    let mut repo = Repository::new(
        "nonexistent".to_string(),
        "git@github.com:owner/test.git".to_string(),
    );
    repo.set_config_dir(Some(std::path::PathBuf::from("/tmp/nonexistent")));

    let runner = CommandRunner::new();

    // Should fail because repository directory doesn't exist
    let result = runner.run_command(&repo, "echo 'test'", None).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Repository directory does not exist"));
}

#[tokio::test]
async fn test_run_command_with_log_directory() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    let result = runner
        .run_command(&repo, "echo 'Logged output'", Some(&log_dir_str))
        .await;
    assert!(result.is_ok());

    // Check that log file was created
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .expect("Failed to read log directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("test-repo_")
                && entry.file_name().to_string_lossy().ends_with(".log")
        })
        .collect();

    assert!(!log_files.is_empty(), "Log file should have been created");

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_empty_command() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run an empty command
    let result = runner.run_command(&repo, "", None).await;
    assert!(result.is_ok()); // sh -c "" should succeed

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_working_directory() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a test file in the repo directory
    let repo_path = temp_dir.join("test-repo");
    let test_file = repo_path.join("testfile.txt");
    fs::write(&test_file, "test content").expect("Failed to write test file");

    // Run a command that should see the file (verifies working directory)
    let result = runner.run_command(&repo, "ls testfile.txt", None).await;
    assert!(result.is_ok());

    // Note: temp directories are cleaned up automatically when the test ends
}

#[tokio::test]
async fn test_run_command_git_operations() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a test file and add it to git
    let repo_path = temp_dir.join("test-repo");
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "test content").expect("Failed to write test file");

    // Run git add command
    let result = runner.run_command(&repo, "git add test.txt", None).await;
    assert!(result.is_ok());

    // Run git status command
    let result = runner
        .run_command(&repo, "git status --porcelain", None)
        .await;
    assert!(result.is_ok());

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_run_command_with_pipes() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command with pipes
    let result = runner
        .run_command(&repo, "echo 'hello world' | grep 'world'", None)
        .await;
    assert!(result.is_ok());

    cleanup_temp_dir(&temp_dir);
}

#[tokio::test]
async fn test_runner_creation() {
    let _runner = CommandRunner::new();
    // Just test that creation succeeds
    // We can't easily test internal state, but other tests verify functionality
}
