//! Comprehensive integration tests for the git module.

use repos::{
    config::Repository,
    git::{
        Logger, add_all_changes, clone_repository, commit_changes, create_and_checkout_branch,
        get_default_branch, has_changes, push_branch, remove_repository,
    },
};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// =================================
// ===== Helper Functions
// =================================

/// Helper function to create a git repository in a directory for testing.
fn create_git_repo(path: &Path, remote_url: Option<&str>) -> std::io::Result<()> {
    Command::new("git").arg("init").current_dir(path).output()?;
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;
    fs::write(path.join("README.md"), "# Test Repository")?;
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;
    if let Some(url) = remote_url {
        Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(path)
            .output()?;
    }
    Ok(())
}

/// Helper function to create a test repository config object.
fn create_test_repository(name: &str, url: &str, path: Option<String>) -> Repository {
    Repository {
        name: name.to_string(),
        url: url.to_string(),
        tags: vec!["test".to_string()],
        path,
        branch: None,
        config_dir: None,
    }
}

// =================================
// ===== Logger Tests
// =================================

#[test]
fn test_logger_methods() {
    let repo = create_test_repository("test-repo", "https://github.com/user/test-repo.git", None);
    let logger = Logger; // Also tests default implementation

    // These tests just ensure the logger methods don't panic.
    logger.info(&repo, "Test info message");
    logger.success(&repo, "Test success message");
    logger.warn(&repo, "Test warning message");
    logger.error(&repo, "Test error message");
}

// =================================
// ===== Clone and Remove Tests
// =================================

#[test]
fn test_clone_repository_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("existing-repo");
    fs::create_dir_all(&target_path).unwrap();

    let repo = Repository {
        name: "existing-repo".to_string(),
        url: "https://github.com/user/existing-repo.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    // Should succeed but skip cloning because the directory exists.
    let result = clone_repository(&repo);
    assert!(result.is_ok());
}

#[test]
fn test_clone_repository_network_failure() {
    use uuid::Uuid;
    let temp_dir = TempDir::new().unwrap();
    let unique_name = format!("network-fail-test-{}", Uuid::new_v4());
    let repo = Repository {
        name: unique_name,
        url: "https://invalid-domain-12345-unique-xyz.com/repo.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    // Ensure the target directory doesn't exist by checking and removing if it does
    let target_dir = repo.get_target_dir();
    if std::path::Path::new(&target_dir).exists() {
        std::fs::remove_dir_all(&target_dir).ok();
    }

    let result = clone_repository(&repo);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to execute git clone command")
            || error_msg.contains("Failed to clone repository")
    );
}

#[test]
fn test_remove_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("to-remove");
    fs::create_dir_all(&repo_path).unwrap();
    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let repo = Repository {
        name: "to-remove".to_string(),
        url: "https://github.com/user/to-remove.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    // Test successful removal
    assert!(repo_path.exists());
    let result = remove_repository(&repo);
    assert!(result.is_ok());
    assert!(!repo_path.exists());

    // Test removal of non-existent directory
    let result_nonexistent = remove_repository(&repo);
    assert!(result_nonexistent.is_err());
    assert!(
        result_nonexistent
            .unwrap_err()
            .to_string()
            .contains("does not exist")
    );
}

// =================================
// ===== State Check Tests
// =================================

#[test]
fn test_has_changes() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Test clean repo
    let result_clean = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result_clean.is_ok());
    assert!(!result_clean.unwrap());

    // Test with untracked file
    fs::write(temp_dir.path().join("new_file.txt"), "new content").unwrap();
    let result_untracked = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result_untracked.is_ok());
    assert!(result_untracked.unwrap());

    // Test with modified file
    fs::write(
        temp_dir.path().join("README.md"),
        "# Modified Test Repository",
    )
    .unwrap();
    let result_modified = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result_modified.is_ok());
    assert!(result_modified.unwrap());

    // Test with staged changes
    Command::new("git")
        .args(["add", "new_file.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let result_staged = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result_staged.is_ok());
    assert!(result_staged.unwrap());
}

#[test]
fn test_has_changes_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

// =================================
// ===== Branching Tests
// =================================

#[test]
fn test_create_and_checkout_branch() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();
    let path_str = temp_dir.path().to_str().unwrap();

    // Test successful creation
    let result = create_and_checkout_branch(path_str, "new-feature");
    assert!(result.is_ok());

    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "new-feature");

    // Test creating an existing branch (should fail)
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let result_exists = create_and_checkout_branch(path_str, "new-feature");
    assert!(result_exists.is_err());

    // Test invalid branch name
    let result_invalid = create_and_checkout_branch(path_str, "invalid..branch");
    assert!(result_invalid.is_err());

    // Test with special characters
    let result_special = create_and_checkout_branch(path_str, "feature/branch-v2");
    assert!(result_special.is_ok());
}

#[test]
fn test_create_and_checkout_branch_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = create_and_checkout_branch(temp_dir.path().to_str().unwrap(), "new-branch");
    assert!(result.is_err());
}

#[test]
fn test_get_default_branch() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Test with remote HEAD set to 'develop'
    Command::new("git")
        .args([
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/develop",
        ])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let result_remote = get_default_branch(temp_dir.path().to_str().unwrap());
    assert!(result_remote.is_ok());
    assert_eq!(result_remote.unwrap(), "develop");

    // Test fallback to current branch
    let temp_dir_no_remote = TempDir::new().unwrap();
    create_git_repo(temp_dir_no_remote.path(), None).unwrap();
    Command::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(temp_dir_no_remote.path())
        .output()
        .unwrap();
    let result_current = get_default_branch(temp_dir_no_remote.path().to_str().unwrap());
    assert!(result_current.is_ok());
    assert_eq!(result_current.unwrap(), "feature-branch");

    // Test fallback with detached HEAD
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_dir_no_remote.path())
        .output()
        .unwrap();
    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Command::new("git")
        .args(["checkout", &commit_hash])
        .current_dir(temp_dir_no_remote.path())
        .output()
        .unwrap();
    let result_detached = get_default_branch(temp_dir_no_remote.path().to_str().unwrap());
    assert!(result_detached.is_ok());
    assert_eq!(result_detached.unwrap(), "main"); // Fallback to 'main'
}

#[test]
fn test_get_default_branch_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    // Non-git directory should use fallback.
    let result = get_default_branch(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "main");
}

// =================================
// ===== Add, Commit, Push Tests
// =================================

#[test]
fn test_add_all_changes() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Test with no changes
    let result_no_changes = add_all_changes(temp_dir.path().to_str().unwrap());
    assert!(result_no_changes.is_ok());

    // Test with new files
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
    let result_with_changes = add_all_changes(temp_dir.path().to_str().unwrap());
    assert!(result_with_changes.is_ok());

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&output.stdout);
    assert!(status.contains("A  file1.txt"));
    assert!(status.contains("A  file2.txt"));
}

#[test]
fn test_add_all_changes_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = add_all_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_commit_changes() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();
    let path_str = temp_dir.path().to_str().unwrap();

    // Test with nothing to commit
    let result_nothing = commit_changes(path_str, "Empty commit");
    assert!(result_nothing.is_err());

    // Test successful commit
    fs::write(temp_dir.path().join("commit_test.txt"), "commit content").unwrap();
    add_all_changes(path_str).unwrap();
    let result_success = commit_changes(path_str, "Test commit message");
    assert!(result_success.is_ok());

    let output = Command::new("git")
        .args(["log", "--oneline", "-n", "1"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Test commit message"));

    // Test commit with special characters
    fs::write(temp_dir.path().join("special.txt"), "special").unwrap();
    add_all_changes(path_str).unwrap();
    let result_special = commit_changes(
        path_str,
        "Test with 'quotes' and \"double quotes\" and émojis 🚀",
    );
    assert!(result_special.is_ok());
    let output_special = Command::new("git")
        .args(["log", "--oneline", "-n", "1"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let log_special = String::from_utf8_lossy(&output_special.stdout);
    assert!(log_special.contains("Test with 'quotes' and \"double quotes\" and émojis 🚀"));
}

#[test]
fn test_commit_changes_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = commit_changes(temp_dir.path().to_str().unwrap(), "Test commit");
    assert!(result.is_err());
}

#[test]
fn test_push_branch() {
    // Test with invalid repo
    let temp_dir_invalid = TempDir::new().unwrap();
    let result_invalid = push_branch(temp_dir_invalid.path().to_str().unwrap(), "main");
    assert!(result_invalid.is_err());

    // Test with no remote
    let temp_dir_no_remote = TempDir::new().unwrap();
    create_git_repo(temp_dir_no_remote.path(), None).unwrap();
    let result_no_remote = push_branch(temp_dir_no_remote.path().to_str().unwrap(), "main");
    assert!(result_no_remote.is_err());

    // Test with a (non-functional) remote
    let temp_dir_with_remote = TempDir::new().unwrap();
    create_git_repo(
        temp_dir_with_remote.path(),
        Some("https://github.com/user/test.git"),
    )
    .unwrap();
    let result_with_remote = push_branch(temp_dir_with_remote.path().to_str().unwrap(), "main");
    assert!(result_with_remote.is_err()); // Expected to fail as the remote isn't real/accessible
    assert!(
        result_with_remote
            .unwrap_err()
            .to_string()
            .contains("Failed to push")
    );
}
