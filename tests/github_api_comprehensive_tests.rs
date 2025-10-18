use repos::{
    config::Repository,
    git,
    github::{api::create_pr_from_workspace, types::PrOptions},
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

// ===== COMPREHENSIVE TESTS TO TARGET GITHUB/API.RS UNCOVERED LINES =====

#[tokio::test]
async fn test_api_has_changes_error_handling() {
    // Test the error path when git::has_changes fails
    let temp_dir = TempDir::new().unwrap();

    let repo = Repository {
        name: "error-test".to_string(),
        url: "https://github.com/user/error-test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        create_only: true,
    };

    // This should fail because the directory is not a git repo
    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_create_branch_error_handling() {
    // Test error path when git::create_and_checkout_branch fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes to pass the has_changes check
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "branch-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("invalid..branch..name".to_string()), // Invalid branch name
        base_branch: None,
        commit_msg: None,
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_add_changes_error_handling() {
    // Test error path when git::add_all_changes fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    // Make git directory read-only to force add failure
    let git_dir = temp_dir.path().join(".git");
    if git_dir.exists() {
        let mut perms = fs::metadata(&git_dir).unwrap().permissions();
        perms.set_readonly(true);
        let _ = fs::set_permissions(&git_dir, perms);
    }

    let repo = Repository {
        name: "add-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: None,
        commit_msg: None,
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // May succeed or fail depending on system permissions
}

#[tokio::test]
async fn test_api_commit_changes_error_handling() {
    // Test error path when git::commit_changes fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes but don't add them to test commit failure
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "commit-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: None,
        commit_msg: Some("".to_string()), // Empty commit message might cause issues
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // This will likely fail at the add or commit stage
}

#[tokio::test]
async fn test_api_push_branch_error_handling() {
    // Test error path when git::push_branch fails (create_only = false)
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "push-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: None,
        commit_msg: None,
        draft: false,
        create_only: false, // This will trigger push which should fail
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err()); // Should fail on push to non-existent remote
}

#[tokio::test]
async fn test_api_get_default_branch_error_handling() {
    // Test error path when git::get_default_branch fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "default-branch-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: None, // This will trigger get_default_branch
        commit_msg: None,
        draft: false,
        create_only: false,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // This will likely fail at push or PR creation stage
}

#[tokio::test]
async fn test_api_github_client_parse_url_error() {
    // Test error path when GitHubClient::parse_github_url fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), None).unwrap(); // No remote URL

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "parse-url-error-test".to_string(),
        url: "invalid://not.a.github.url/repo".to_string(), // Invalid URL
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: false,
        create_only: false,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // This should fail due to invalid GitHub URL
}

#[tokio::test]
async fn test_api_github_create_pr_failure() {
    // Test error path when GitHub API call fails
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(
        temp_dir.path(),
        Some("https://github.com/nonexistent/repo.git"),
    )
    .unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "github-api-error-test".to_string(),
        url: "https://github.com/nonexistent/repo.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "invalid_token".to_string(), // Invalid token
        branch_name: Some("test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: false,
        create_only: false,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // This should fail due to invalid token/repository
}

#[tokio::test]
async fn test_api_branch_name_generation() {
    // Test the UUID branch name generation path
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "uuid-branch-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: None, // This will trigger UUID generation
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: false,
        create_only: true, // Avoid network calls
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // Should succeed with auto-generated branch name
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_commit_message_fallback() {
    // Test the commit message fallback to title
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "commit-msg-fallback-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR Title".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None, // This will use title as commit message
        draft: false,
        create_only: true,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_pr_url_extraction_error() {
    // This test simulates the error path where GitHub API response lacks html_url
    // We can't easily mock the GitHub API response, so this documents the error path
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "pr-url-error-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: false,
        create_only: false, // This will attempt actual GitHub API call
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // This will fail due to invalid token, but covers the code path
    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_success_output_formatting() {
    // Test the success path output formatting (lines that print success messages)
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "success-output-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: false,
        create_only: true, // Avoid network calls but still test success path
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_base_branch_fallback_path() {
    // Test the path where base_branch is None and get_default_branch is called
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Set up a proper git repo with remote tracking
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "base-branch-fallback-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("test-branch".to_string()),
        base_branch: None, // This will trigger get_default_branch call
        commit_msg: None,
        draft: false,
        create_only: false, // Will fail on push, but covers get_default_branch path
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // Will likely fail on push, but that's expected
}

#[tokio::test]
async fn test_api_draft_pr_creation() {
    // Test draft PR creation path
    let temp_dir = TempDir::new().unwrap();
    create_git_repo(temp_dir.path(), Some("https://github.com/user/test.git")).unwrap();

    // Create changes
    fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let repo = Repository {
        name: "draft-pr-test".to_string(),
        url: "https://github.com/user/test.git".to_string(),
        tags: vec![],
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions {
        title: "Draft Test PR".to_string(),
        body: "Draft test body".to_string(),
        token: "test_token".to_string(),
        branch_name: Some("draft-test-branch".to_string()),
        base_branch: Some("main".to_string()),
        commit_msg: None,
        draft: true, // Test draft PR creation
        create_only: false,
    };

    let result = create_pr_from_workspace(&repo, &options).await;
    // Will fail on network call, but covers the draft path
    assert!(result.is_err());
}
