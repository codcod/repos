use repos::config::repository::Repository;
use repos::github::api::create_pr_from_workspace;
use repos::github::types::PrOptions;
use repos_github::GitHubClient;
use std::fs;
use tempfile::TempDir;

// ===== GitHub Client Tests =====

#[test]
fn test_github_client_new_with_token() {
    let _client = GitHubClient::new(Some("test_token".to_string()));
    // Client should be created successfully (we can't test internal state)
    // This tests the constructor
}

#[test]
fn test_github_client_new_without_token() {
    let _client = GitHubClient::new(None);
    // Client should be created successfully without token
    // This tests the constructor
}

// Note: parse_github_url is now an internal function in api.rs
// URL parsing is tested indirectly through the PR workflow tests

// ===== GitHub API Integration Tests =====

/// Helper function to create a git repository in a directory
fn create_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    // Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()?;

    // Configure git (required for commits)
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Ok(())
}

#[tokio::test]
async fn test_create_pr_from_workspace_with_changes_success_flow() {
    // Setup temporary directory with real git repo structure
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    create_git_repo(&repo_path).unwrap();

    // Create a file and commit
    fs::write(repo_path.join("test.txt"), "test content").unwrap();

    // Add and commit initial file
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    // Create new changes to test with
    fs::write(repo_path.join("changes.txt"), "new changes").unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    )
    .create_only();

    // This should succeed and create a branch without network calls
    let result = create_pr_from_workspace(&repo, &options).await;

    // Should succeed since we're in create_only mode
    assert!(result.is_ok());

    // Verify branch was created
    let output = std::process::Command::new("git")
        .args(["branch", "--list"])
        .current_dir(&repo_path)
        .output()
        .expect("git branch failed");

    let branches = String::from_utf8(output.stdout).unwrap();
    assert!(branches.contains("automated-changes-") || branches.contains("* automated-changes-"));
}

#[tokio::test]
async fn test_create_pr_workspace_no_changes_early_return() {
    // Setup temporary directory with clean git repo
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    create_git_repo(&repo_path).unwrap();

    // Create and commit initial file to have a clean repo
    fs::write(repo_path.join("initial.txt"), "initial").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    );

    // This should hit the early return path for no changes
    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_pr_workspace_commit_message_fallback() {
    // Setup temporary directory with changes
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    create_git_repo(&repo_path).unwrap();

    // Create initial commit
    fs::write(repo_path.join("initial.txt"), "initial").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    // Create changes
    fs::write(repo_path.join("changes.txt"), "new changes").unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    // Options without commit_msg to test fallback to title
    let options = PrOptions::new(
        "Test PR Title".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    )
    .create_only();

    // This should use title as commit message (fallback path)
    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Check that the commit was made with the title
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .current_dir(&repo_path)
        .output()
        .expect("git log failed");

    let commit_msg = String::from_utf8(output.stdout).unwrap();
    assert_eq!(commit_msg, "Test PR Title");
}

#[tokio::test]
async fn test_create_pr_workspace_branch_name_generation() {
    // Setup temporary directory with changes
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    create_git_repo(&repo_path).unwrap();

    // Create initial commit
    fs::write(repo_path.join("initial.txt"), "initial").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    // Create changes
    fs::write(repo_path.join("changes.txt"), "new changes").unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    // Options without branch_name to test auto-generation
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    )
    .create_only();

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Verify a feature branch was created
    let output = std::process::Command::new("git")
        .args(["branch", "--list"])
        .current_dir(&repo_path)
        .output()
        .expect("git branch failed");

    let branches = String::from_utf8(output.stdout).unwrap();
    assert!(branches.contains("automated-changes-") || branches.contains("* automated-changes-"));
}

#[tokio::test]
async fn test_create_pr_workspace_git_operations_error_paths() {
    // Setup temporary directory but intentionally break git operations
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Don't initialize git repo to trigger git errors
    fs::write(repo_path.join("changes.txt"), "changes").unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    )
    .create_only();

    // This should fail on git::has_changes due to no git repo
    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_pr_workspace_custom_branch_and_commit() {
    // Setup temporary directory with changes
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    create_git_repo(&repo_path).unwrap();

    // Create initial commit
    fs::write(repo_path.join("initial.txt"), "initial").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    // Create changes
    fs::write(repo_path.join("changes.txt"), "new changes").unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/owner/repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    // Options with custom branch name and commit message
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    )
    .with_branch_name("custom-branch".to_string())
    .with_commit_message("Custom commit message".to_string())
    .create_only();

    let result = create_pr_from_workspace(&repo, &options).await;
    assert!(result.is_ok());

    // Verify custom branch was created
    let output = std::process::Command::new("git")
        .args(["branch", "--list"])
        .current_dir(&repo_path)
        .output()
        .expect("git branch failed");

    let branches = String::from_utf8(output.stdout).unwrap();
    assert!(branches.contains("custom-branch"));

    // Verify custom commit message was used
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .current_dir(&repo_path)
        .output()
        .expect("git log failed");

    let commit_msg = String::from_utf8(output.stdout).unwrap();
    assert_eq!(commit_msg, "Custom commit message");
}

// ===== GitHub End-to-End Integration Tests =====

#[tokio::test]
async fn test_github_integration_auth_client_api() {
    // Test complete integration flow with client and API
    let token = "ghp_integration_test_token".to_string();
    let _client = GitHubClient::new(Some(token.clone()));

    // Setup git repo for API testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    create_git_repo(&repo_path).unwrap();

    // Create initial commit
    fs::write(repo_path.join("integration.txt"), "integration test").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("git add failed");

    std::process::Command::new("git")
        .args(["commit", "-m", "Integration commit"])
        .current_dir(&repo_path)
        .output()
        .expect("git commit failed");

    // Create changes for PR
    fs::write(repo_path.join("changes.txt"), "integration changes").unwrap();

    let repository = Repository {
        name: "integration-repo".to_string(),
        url: "https://github.com/owner/integration-repo.git".to_string(),
        path: Some(repo_path.to_string_lossy().to_string()),
        tags: Vec::new(),
        branch: None,
        config_dir: None,
    };

    let options = PrOptions::new(
        "Integration Test PR".to_string(),
        "This PR tests the integration flow".to_string(),
        token,
    )
    .create_only();

    // Test the complete flow
    let result = create_pr_from_workspace(&repository, &options).await;
    assert!(result.is_ok());
}
