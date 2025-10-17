//! GitHub API tests focusing on PR creation functionality

use repos::config::Repository;
use repos::github::api::create_pull_request;
use repos::github::types::PrOptions;
use std::fs;
use std::path::PathBuf;

/// Helper function to create a test repository
fn create_test_repo(name: &str, url: &str) -> (Repository, PathBuf) {
    let temp_base = std::env::temp_dir();
    let unique_id = format!("{}-{}", name, std::process::id());
    let temp_dir = temp_base.join(format!("repos_test_{}", unique_id));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Create the actual repository directory
    let repo_dir = temp_dir.join(name);
    fs::create_dir_all(&repo_dir).expect("Failed to create repo directory");

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_dir)
        .output()
        .expect("Failed to initialize git repository");

    // Configure git user for testing
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_dir)
        .output()
        .expect("Failed to configure git user");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_dir)
        .output()
        .expect("Failed to configure git email");

    let mut repo = Repository::new(name.to_string(), url.to_string());
    repo.set_config_dir(Some(temp_dir.clone()));

    (repo, temp_dir)
}

#[tokio::test]
async fn test_create_pull_request_no_changes() {
    let (repo, _temp_dir) = create_test_repo("test-repo", "git@github.com:owner/test-repo.git");

    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "fake-token".to_string(),
    );

    // Should succeed with no changes (just prints a message and returns Ok)
    let result = create_pull_request(&repo, &options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_options_builder() {
    let options = PrOptions::new(
        "Test Title".to_string(),
        "Test Body".to_string(),
        "test-token".to_string(),
    )
    .with_branch_name("feature-branch".to_string())
    .with_base_branch("develop".to_string())
    .with_commit_message("Custom commit".to_string())
    .as_draft()
    .create_only();

    assert_eq!(options.title, "Test Title");
    assert_eq!(options.body, "Test Body");
    assert_eq!(options.token, "test-token");
    assert_eq!(options.branch_name, Some("feature-branch".to_string()));
    assert_eq!(options.base_branch, Some("develop".to_string()));
    assert_eq!(options.commit_msg, Some("Custom commit".to_string()));
    assert!(options.draft);
    assert!(options.create_only);
}

#[tokio::test]
async fn test_pr_options_defaults() {
    let options = PrOptions::new(
        "Test Title".to_string(),
        "Test Body".to_string(),
        "test-token".to_string(),
    );

    assert_eq!(options.title, "Test Title");
    assert_eq!(options.body, "Test Body");
    assert_eq!(options.token, "test-token");
    assert_eq!(options.branch_name, None);
    assert_eq!(options.base_branch, None);
    assert_eq!(options.commit_msg, None);
    assert!(!options.draft);
    assert!(!options.create_only);
}
