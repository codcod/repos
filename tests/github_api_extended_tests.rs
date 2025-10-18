use repos::{config::Repository, github::api::create_pr_from_workspace, github::types::PrOptions};
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

/// Helper function to create a git repository in a directory

#[tokio::test]
async fn test_create_pr_from_workspace_no_changes() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-branch".to_string()),
        commit_msg: Some("Test commit".to_string()),
        draft: false,
        create_only: false,
    };

    // Should succeed but not create PR since no changes
    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_pr_from_workspace_with_changes_create_only() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(temp_dir.path().join("new_file.txt"), "new content").unwrap();

    let repo = Repository {
        name: "test-repo-changes".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR with changes".to_string(),
        body: "Test body with changes".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-branch-changes".to_string()),
        commit_msg: Some("Test commit with changes".to_string()),
        draft: false,
        create_only: true, // Only create branch/commit, don't push/create PR
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Verify branch was created
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let current_branch_output = String::from_utf8_lossy(&output.stdout);
    let current_branch = current_branch_output.trim();
    assert_eq!(current_branch, "test-branch-changes");
}

#[tokio::test]
async fn test_create_pr_from_workspace_auto_branch_name() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(
        temp_dir.path().join("auto_branch_file.txt"),
        "auto branch content",
    )
    .unwrap();

    let repo = Repository {
        name: "test-repo-auto-branch".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR auto branch".to_string(),
        body: "Test body auto branch".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: None, // Let it auto-generate
        commit_msg: Some("Test commit auto branch".to_string()),
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Verify a branch was created (should start with "automated-changes")
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let current_branch_output = String::from_utf8_lossy(&output.stdout);
    let current_branch = current_branch_output.trim();
    assert!(current_branch.starts_with("automated-changes"));
}

#[tokio::test]
async fn test_create_pr_from_workspace_auto_commit_message() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(
        temp_dir.path().join("auto_commit_file.txt"),
        "auto commit content",
    )
    .unwrap();

    let repo = Repository {
        name: "test-repo-auto-commit".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR auto commit message".to_string(),
        body: "Test body auto commit".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-auto-commit".to_string()),
        commit_msg: None, // Should use title as commit message
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Verify commit was made with title as message
    let output = Command::new("git")
        .args(["log", "--oneline", "-n", "1"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let log_output = String::from_utf8_lossy(&output.stdout);
    assert!(log_output.contains("Test PR auto commit message"));
}

#[tokio::test]
async fn test_create_pr_from_workspace_draft_mode() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(temp_dir.path().join("draft_file.txt"), "draft content").unwrap();

    let repo = Repository {
        name: "test-repo-draft".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test Draft PR".to_string(),
        body: "Test draft body".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-draft-branch".to_string()),
        commit_msg: Some("Test draft commit".to_string()),
        draft: true, // Draft mode
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_pr_from_workspace_no_base_branch() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(temp_dir.path().join("no_base_file.txt"), "no base content").unwrap();

    let repo = Repository {
        name: "test-repo-no-base".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR no base".to_string(),
        body: "Test body no base".to_string(),
        token: "test-token".to_string(),
        base_branch: None, // Should auto-detect
        branch_name: Some("test-no-base-branch".to_string()),
        commit_msg: Some("Test no base commit".to_string()),
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_pr_from_workspace_git_operations_failure() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize as git repo to cause git operation failures

    let repo = Repository {
        name: "test-repo-fail".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR fail".to_string(),
        body: "Test body fail".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-fail-branch".to_string()),
        commit_msg: Some("Test fail commit".to_string()),
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err()); // Should fail due to invalid git repo
}

#[tokio::test]
async fn test_create_pr_from_workspace_empty_options() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(
        temp_dir.path().join("empty_options_file.txt"),
        "empty options content",
    )
    .unwrap();

    let repo = Repository {
        name: "test-repo-empty".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "".to_string(), // Empty title
        body: "".to_string(),  // Empty body
        token: "test-token".to_string(),
        base_branch: None,
        branch_name: None,
        commit_msg: None,
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // Empty title might cause an error, which is expected behavior
    if result.is_err() {
        let error_msg = format!("{:?}", result.err().unwrap());
        assert!(
            error_msg.contains("title")
                || error_msg.contains("empty")
                || error_msg.contains("required")
        );
    } else {
        // If it succeeds, that's also fine
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_create_pr_from_workspace_invalid_repo_url() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("invalid-url")).unwrap();

    // Add a new file to create changes
    fs::write(
        temp_dir.path().join("invalid_url_file.txt"),
        "invalid url content",
    )
    .unwrap();

    let repo = Repository {
        name: "test-repo-invalid-url".to_string(),
        url: "invalid-github-url".to_string(), // Invalid URL format
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR invalid URL".to_string(),
        body: "Test body invalid URL".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-invalid-url-branch".to_string()),
        commit_msg: Some("Test invalid URL commit".to_string()),
        draft: false,
        create_only: false, // Try to create actual PR (will fail)
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err()); // Should fail due to invalid URL when trying to create PR
}

#[test]
fn test_pr_options_validation() {
    let options = PrOptions {
        title: "Valid Title".to_string(),
        body: "Valid Body".to_string(),
        token: "valid-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("valid-branch".to_string()),
        commit_msg: Some("Valid commit".to_string()),
        draft: false,
        create_only: false,
    };

    // Test that options can be created and accessed
    assert_eq!(options.title, "Valid Title");
    assert_eq!(options.body, "Valid Body");
    assert_eq!(options.token, "valid-token");
    assert_eq!(options.base_branch, Some("main".to_string()));
    assert_eq!(options.branch_name, Some("valid-branch".to_string()));
    assert_eq!(options.commit_msg, Some("Valid commit".to_string()));
    assert!(!options.draft);
    assert!(!options.create_only);
}

#[test]
fn test_pr_options_defaults() {
    let options = PrOptions {
        title: "Title".to_string(),
        body: "Body".to_string(),
        token: "token".to_string(),
        base_branch: None,
        branch_name: None,
        commit_msg: None,
        draft: false,
        create_only: false,
    };

    // Test that None options work correctly
    assert!(options.base_branch.is_none());
    assert!(options.branch_name.is_none());
    assert!(options.commit_msg.is_none());
}

#[tokio::test]
async fn test_create_pr_from_workspace_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Add a new file to create changes
    fs::write(
        temp_dir.path().join("special_chars_file.txt"),
        "special chars content",
    )
    .unwrap();

    let repo = Repository {
        name: "test-repo-special".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR with 'quotes' and \"double quotes\" & symbols!".to_string(),
        body: "Test body with special chars: @#$%^&*()[]{}".to_string(),
        token: "test-token".to_string(),
        base_branch: Some("main".to_string()),
        branch_name: Some("test-special-chars".to_string()),
        commit_msg: Some("Test commit with special chars: <>&".to_string()),
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}
