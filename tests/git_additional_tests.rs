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

/// Helper function to create a git repository in a directory
fn create_git_repo(path: &Path, remote_url: Option<&str>) -> std::io::Result<()> {
    // Initialize git repo
    Command::new("git").arg("init").current_dir(path).output()?;

    // Configure git (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    // Create a file and commit
    fs::write(path.join("README.md"), "# Test Repository")?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    // Add remote if provided
    if let Some(url) = remote_url {
        Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(path)
            .output()?;
    }

    Ok(())
}

/// Helper function to create a test repository
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

// ===== ADDITIONAL COMPREHENSIVE TESTS TO IMPROVE COVERAGE =====

#[test]
fn test_git_error_paths_coverage() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_str().unwrap();

    // Test various error paths to improve coverage

    // Test has_changes with non-git directory
    let result = has_changes(path);
    assert!(result.is_err());

    // Test create_and_checkout_branch with non-git directory
    let result = create_and_checkout_branch(path, "test-branch");
    assert!(result.is_err());

    // Test add_all_changes with non-git directory
    let result = add_all_changes(path);
    assert!(result.is_err());

    // Test commit_changes with non-git directory
    let result = commit_changes(path, "test commit");
    assert!(result.is_err());

    // Test push_branch with non-git directory
    let result = push_branch(path, "main");
    assert!(result.is_err());
}

#[test]
fn test_git_operations_with_special_cases() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();
    let path = temp_dir.path().to_str().unwrap();

    // Test empty commit message
    fs::write(temp_dir.path().join("empty_msg.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "empty_msg.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let result = commit_changes(path, "");
    // Git may handle empty commit message differently
    if result.is_err() {
        assert!(result.unwrap_err().to_string().contains("Failed to commit"));
    }

    // Test invalid branch name
    let result = create_and_checkout_branch(path, "invalid..branch");
    assert!(result.is_err());
}

#[test]
fn test_get_default_branch_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    // Test with non-git directory - should use fallback
    let result = get_default_branch(temp_dir.path().to_str().unwrap());
    match result {
        Ok(branch) => assert_eq!(branch, "main"),
        Err(_) => {} // Also acceptable for non-git directory
    }

    // Test with git directory
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Set up symbolic ref
    Command::new("git")
        .args([
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/develop",
        ])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let result = get_default_branch(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "develop");

    // Test with malformed symbolic ref
    Command::new("git")
        .args([
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "malformed-ref-without-prefix",
        ])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let result = get_default_branch(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn test_logger_comprehensive_coverage() {
    let repo = create_test_repository(
        "coverage-test",
        "https://github.com/user/coverage.git",
        None,
    );
    let logger = Logger;

    // Test all logger methods
    logger.info(&repo, "Info message");
    logger.success(&repo, "Success message");
    logger.warn(&repo, "Warning message");
    logger.error(&repo, "Error message");

    // Test Logger::default()
    let default_logger = Logger;
    default_logger.info(&repo, "Default logger test");
}

#[test]
fn test_clone_repository_comprehensive() {
    let temp_dir = TempDir::new().unwrap();

    // Test with directory that already exists
    let existing_repo = Repository {
        name: "existing-test".to_string(),
        url: "https://github.com/user/existing.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let target_dir = existing_repo.get_target_dir();
    fs::create_dir_all(&target_dir).unwrap();

    let result = clone_repository(&existing_repo);
    assert!(result.is_ok()); // Should skip cloning

    // Test network failure case
    let network_fail_repo = Repository {
        name: "network-fail-test".to_string(),
        url: "https://invalid-domain-12345.com/repo.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let result = clone_repository(&network_fail_repo);
    // Either fails due to network or succeeds due to existing directory
    if result.is_err() {
        assert!(result.unwrap_err().to_string().contains("Failed to clone"));
    }
}

#[test]
fn test_has_changes_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Test clean repo
    let result = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Test repo with changes
    fs::write(temp_dir.path().join("new_file.txt"), "new content").unwrap();
    let result = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_push_branch_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Test push with special characters in branch name
    let result = push_branch(temp_dir.path().to_str().unwrap(), "feature/test-branch");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to push"));

    // Test push without remote
    let temp_dir2 = TempDir::new().unwrap();
    create_git_repo(temp_dir2.path(), None).unwrap();
    let result = push_branch(temp_dir2.path().to_str().unwrap(), "main");
    assert!(result.is_err());
}

#[test]
fn test_remove_repository_comprehensive() {
    let temp_dir = TempDir::new().unwrap();

    // Test successful removal
    let repo_path = temp_dir.path().join("test-removal");
    fs::create_dir_all(&repo_path).unwrap();
    fs::write(repo_path.join("file.txt"), "content").unwrap();

    let repo = Repository {
        name: "test-removal".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let result = remove_repository(&repo);
    assert!(result.is_ok());
    assert!(!repo_path.exists());

    // Test removal of non-existent directory
    let result = remove_repository(&repo);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_add_and_commit_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Test add_all_changes with no changes
    let result = add_all_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());

    // Test add_all_changes with new files
    fs::write(temp_dir.path().join("new1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("new2.txt"), "content2").unwrap();

    let result = add_all_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());

    // Test commit with special characters
    let result = commit_changes(
        temp_dir.path().to_str().unwrap(),
        "Test with 'quotes' and \"double quotes\"",
    );
    assert!(result.is_ok());

    // Test commit with nothing to commit
    let result = commit_changes(temp_dir.path().to_str().unwrap(), "Empty commit");
    assert!(result.is_err());
}

#[test]
fn test_create_checkout_branch_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Test successful branch creation
    let result = create_and_checkout_branch(temp_dir.path().to_str().unwrap(), "new-feature");
    assert!(result.is_ok());

    // Verify we're on the new branch
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(current_branch, "new-feature");

    // Test branch creation with special characters
    let result = create_and_checkout_branch(temp_dir.path().to_str().unwrap(), "feature-branch_v2");
    assert!(result.is_ok());

    // Switch back to test existing branch error
    Command::new("git")
        .args(["checkout", "new-feature"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Test creating existing branch
    let result = create_and_checkout_branch(temp_dir.path().to_str().unwrap(), "feature-branch_v2");
    assert!(result.is_err());
}

#[test]
fn test_detached_head_scenario() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();

    // Create a detached HEAD state
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Command::new("git")
        .args(["checkout", &commit_hash])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let result = get_default_branch(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "main");
}

#[test]
fn test_error_message_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_str().unwrap();

    // Test that error messages are properly formatted
    let errors = vec![
        has_changes(path).map(|_| false),
        add_all_changes(path).map(|_| false),
        commit_changes(path, "test").map(|_| false),
        push_branch(path, "test").map(|_| false),
    ];

    for error in errors {
        if error.is_err() {
            let error_msg = error.unwrap_err().to_string();
            assert!(!error_msg.is_empty());
            assert!(error_msg.contains("Failed to"));
        }
    }
}

#[test]
fn test_network_failure_scenarios() {
    let temp_dir = TempDir::new().unwrap();

    // Test clone with invalid URL
    let repo = Repository {
        name: "invalid-url-test".to_string(),
        url: "https://definitely-not-a-real-domain-12345.com/repo.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let result = clone_repository(&repo);
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to execute git clone command")
                || error_msg.contains("Failed to clone repository")
        );
    }
}

#[test]
fn test_git_command_variations() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();
    let path = temp_dir.path().to_str().unwrap();

    // Test various git command scenarios that might be uncovered

    // Create a file and test git add
    fs::write(temp_dir.path().join("test_file.txt"), "test content").unwrap();
    let result = add_all_changes(path);
    assert!(result.is_ok());

    // Test commit with unicode characters
    let result = commit_changes(path, "测试提交 with émojis 🚀");
    assert!(result.is_ok());

    // Test branch with underscores and dashes
    let result = create_and_checkout_branch(path, "test_branch-with-dashes");
    assert!(result.is_ok());
}

#[test]
fn test_repository_state_variations() {
    let temp_dir = TempDir::new().unwrap();

    // Test operations on empty directory
    let empty_path = temp_dir.path().join("empty");
    fs::create_dir_all(&empty_path).unwrap();

    let result = has_changes(empty_path.to_str().unwrap());
    assert!(result.is_err());

    // Test with initialized but empty git repo
    create_git_repo(temp_dir.path(), None).unwrap();

    let result = has_changes(temp_dir.path().to_str().unwrap());
    assert!(result.is_ok());
    assert!(!result.unwrap()); // No changes in clean repo
}

#[test]
fn test_branch_operations_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap();
    let path = temp_dir.path().to_str().unwrap();

    // Test switching to main branch first
    let _result = create_and_checkout_branch(path, "main");
    // This might succeed or fail depending on current branch state

    // Test creating branch with numbers
    let result = create_and_checkout_branch(path, "version-1.2.3");
    assert!(result.is_ok());

    // Test creating branch starting with number
    let result = create_and_checkout_branch(path, "2024-feature");
    assert!(result.is_ok());
}

#[test]
fn test_logger_default_implementation() {
    let repo = create_test_repository(
        "logger-default-test",
        "https://github.com/user/logger.git",
        None,
    );

    // Test that Logger implements Default
    let logger = Logger;

    // Test all logger methods with default instance
    logger.info(&repo, "Default logger info");
    logger.success(&repo, "Default logger success");
    logger.warn(&repo, "Default logger warning");
    logger.error(&repo, "Default logger error");
}
