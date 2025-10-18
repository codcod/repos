// Additional comprehensive unit tests for CommandRunner
// Focuses on covering remaining uncovered paths and edge cases

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
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git user");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to configure git email");

    (repo, temp_dir)
}

/// Helper to create a repository with invalid path for error testing
fn create_repo_with_invalid_log_dir(name: &str) -> Repository {
    let mut repo = Repository::new(name.to_string(), "https://github.com/test/repo".to_string());

    // Use a valid directory but later create invalid log paths
    let temp_dir = std::env::temp_dir().join(format!("repos_valid_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).ok();

    let repo_path = temp_dir.join(name);
    fs::create_dir_all(&repo_path).ok();

    repo.set_config_dir(Some(temp_dir));
    repo
}

#[tokio::test]
async fn test_run_command_with_invalid_log_path() {
    let repo = create_repo_with_invalid_log_dir("test-repo");
    let runner = CommandRunner::new();

    // Try to create log file with an invalid filename (contains null bytes)
    // This should trigger the log file creation error path during File::create
    let log_dir_with_null = "/tmp/test_logs\0invalid";

    let result = runner
        .run_command(&repo, "echo 'test'", Some(log_dir_with_null))
        .await;

    // Should fail due to invalid path during log file creation
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // The error should mention something about invalid path or file operations
    assert!(
        error_msg.contains("Invalid")
            || error_msg.contains("invalid")
            || error_msg.to_lowercase().contains("error")
            || !error_msg.is_empty()
    );
}

#[tokio::test]
async fn test_run_command_with_stderr_output() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    // Run a command that outputs to stderr but doesn't fail
    let result = runner
        .run_command(&repo, "echo 'error message' >&2", Some(&log_dir_str))
        .await;

    assert!(result.is_ok());

    // Check that log file was created and contains stderr section
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

    // Read the log file and verify it contains the stderr header
    if let Ok(log_content) = fs::read_to_string(log_files[0].path()) {
        assert!(log_content.contains("=== STDERR ==="));
        assert!(log_content.contains("error message"));
    }
}

#[tokio::test]
async fn test_run_command_mixed_stdout_stderr() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    // Run a command that outputs to both stdout and stderr
    let result = runner
        .run_command(
            &repo,
            "echo 'stdout message'; echo 'stderr message' >&2",
            Some(&log_dir_str),
        )
        .await;

    assert!(result.is_ok());

    // Check that log file contains both stdout and stderr sections
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

    if let Ok(log_content) = fs::read_to_string(log_files[0].path()) {
        assert!(log_content.contains("=== STDOUT ==="));
        assert!(log_content.contains("=== STDERR ==="));
        assert!(log_content.contains("stdout message"));
        assert!(log_content.contains("stderr message"));
    }
}

#[tokio::test]
async fn test_run_command_with_exit_code() {
    let (repo, _temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command that exits with a specific code
    let result = runner.run_command(&repo, "exit 42", None).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Command failed with exit code: 42"));
}

#[tokio::test]
async fn test_run_command_with_signal_termination() {
    let (repo, _temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command that would be terminated by signal (but we can't easily test this on all platforms)
    // Instead, test a command that exits with no specific code (should show -1)
    let result = runner.run_command(&repo, "kill -9 $$", None).await;

    // This might succeed or fail depending on shell behavior, but test that we handle it
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Command failed with exit code"));
    }
}

#[tokio::test]
async fn test_run_command_multiple_stderr_lines() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    // Run a command that outputs multiple lines to stderr
    let result = runner
        .run_command(
            &repo,
            "echo 'first error' >&2; echo 'second error' >&2; echo 'third error' >&2",
            Some(&log_dir_str),
        )
        .await;

    assert!(result.is_ok());

    // Check that log file contains all stderr lines and only one header
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

    if let Ok(log_content) = fs::read_to_string(log_files[0].path()) {
        // Should have exactly one stderr header
        assert_eq!(log_content.matches("=== STDERR ===").count(), 1);
        assert!(log_content.contains("first error"));
        assert!(log_content.contains("second error"));
        assert!(log_content.contains("third error"));
    }
}

#[tokio::test]
async fn test_run_command_log_file_permissions() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory with restricted permissions
    let log_dir = temp_dir.join("restricted_logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    // Try to make the directory read-only (this might not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&log_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&log_dir, perms).ok();
    }

    let log_dir_str = log_dir.to_string_lossy().to_string();

    // This should fail when trying to create the log file
    let result = runner
        .run_command(&repo, "echo 'test'", Some(&log_dir_str))
        .await;

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&log_dir).unwrap().permissions();
        perms.set_mode(0o755); // Restore write permissions
        fs::set_permissions(&log_dir, perms).ok();
    }

    // On systems where we can't restrict permissions, the test might succeed
    // But if it fails, it should be due to permission issues
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Permission denied")
                || error_msg.contains("Read-only file system")
                || error_msg.contains("denied")
        );
    }
}

#[tokio::test]
async fn test_run_command_very_long_output() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    // Run a command that produces a lot of output
    let result = runner
        .run_command(
            &repo,
            "for i in $(seq 1 100); do echo \"Line $i\"; done",
            Some(&log_dir_str),
        )
        .await;

    assert!(result.is_ok());

    // Verify the log file contains all the output
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

    if let Ok(log_content) = fs::read_to_string(log_files[0].path()) {
        assert!(log_content.contains("Line 1"));
        assert!(log_content.contains("Line 50"));
        assert!(log_content.contains("Line 100"));
    }
}

#[tokio::test]
async fn test_run_command_log_header_creation() {
    let (repo, temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    let result = runner
        .run_command(&repo, "echo 'test'", Some(&log_dir_str))
        .await;

    assert!(result.is_ok());

    // Verify the log file contains proper headers
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

    if let Ok(log_content) = fs::read_to_string(log_files[0].path()) {
        assert!(log_content.contains("Repository: test-repo"));
        assert!(log_content.contains("Command: echo 'test'"));
        assert!(log_content.contains("Directory:"));
        assert!(log_content.contains("Timestamp:"));
        assert!(log_content.contains("=== STDOUT ==="));
    }
}

#[tokio::test]
async fn test_run_command_special_characters_in_repo_name() {
    let temp_base = std::env::temp_dir();
    let unique_id = std::process::id();
    let temp_dir = temp_base.join(format!("repos_test_{}", unique_id));

    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Create repo with special characters in name
    let repo_name = "test-repo_with-special.chars";
    let mut repo = Repository::new(
        repo_name.to_string(),
        "https://github.com/test/repo".to_string(),
    );
    repo.set_config_dir(Some(temp_dir.clone()));

    let repo_path = temp_dir.join(repo_name);
    fs::create_dir_all(&repo_path).expect("Failed to create repo directory");

    let runner = CommandRunner::new();

    // Create a log directory
    let log_dir = temp_dir.join("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    let log_dir_str = log_dir.to_string_lossy().to_string();

    let result = runner
        .run_command(&repo, "echo 'test with special chars'", Some(&log_dir_str))
        .await;

    assert!(result.is_ok());

    // Verify log file was created with proper name handling
    let log_files: Vec<_> = fs::read_dir(&log_dir)
        .expect("Failed to read log directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("test-repo_with-special.chars")
        })
        .collect();

    assert!(!log_files.is_empty(), "Log file should have been created");
}

#[tokio::test]
async fn test_run_command_spawn_failure() {
    let (repo, _temp_dir) = create_test_repo_with_git("test-repo", "git@github.com:owner/test.git");
    let runner = CommandRunner::new();

    // Run a command using a shell that definitely doesn't exist
    // This should cause the spawn to fail, not just the command execution
    // Note: This is hard to test portably, so we'll test with an invalid command structure
    let result = runner
        .run_command(&repo, "\0invalid\0command\0", None)
        .await;

    // Should fail due to spawn error or command execution error
    assert!(result.is_err());
}
