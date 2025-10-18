// Comprehensive unit tests for GitHub types and data structures
// Tests cover struct creation, builder patterns, error handling, and display implementations

use repos::github::types::{
    GitHubError, GitHubRepo, PrOptions, PullRequest, PullRequestParams, User,
};
use serde_json;
use std::error::Error;

#[test]
fn test_pull_request_params_creation() {
    let params = PullRequestParams::new(
        "owner",
        "repo",
        "Test Title",
        "Test body",
        "feature-branch",
        "main",
        false,
    );

    assert_eq!(params.owner, "owner");
    assert_eq!(params.repo, "repo");
    assert_eq!(params.title, "Test Title");
    assert_eq!(params.body, "Test body");
    assert_eq!(params.head, "feature-branch");
    assert_eq!(params.base, "main");
    assert_eq!(params.draft, false);
}

#[test]
fn test_pull_request_params_with_draft() {
    let params = PullRequestParams::new(
        "owner",
        "repo",
        "Draft PR",
        "Draft body",
        "draft-branch",
        "develop",
        true,
    );

    assert_eq!(params.draft, true);
    assert_eq!(params.base, "develop");
}

#[test]
fn test_pr_options_creation() {
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "ghp_token123".to_string(),
    );

    assert_eq!(options.title, "Test PR");
    assert_eq!(options.body, "Test body");
    assert_eq!(options.token, "ghp_token123");
    assert_eq!(options.branch_name, None);
    assert_eq!(options.base_branch, None);
    assert_eq!(options.commit_msg, None);
    assert_eq!(options.draft, false);
    assert_eq!(options.create_only, false);
}

#[test]
fn test_pr_options_builder_with_branch_name() {
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "token".to_string(),
    )
    .with_branch_name("feature/new-feature".to_string());

    assert_eq!(options.branch_name, Some("feature/new-feature".to_string()));
    assert_eq!(options.draft, false);
    assert_eq!(options.create_only, false);
}

#[test]
fn test_pr_options_builder_with_base_branch() {
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "token".to_string(),
    )
    .with_base_branch("develop".to_string());

    assert_eq!(options.base_branch, Some("develop".to_string()));
}

#[test]
fn test_pr_options_builder_with_commit_message() {
    let options = PrOptions::new(
        "Test PR".to_string(),
        "Test body".to_string(),
        "token".to_string(),
    )
    .with_commit_message("Custom commit message".to_string());

    assert_eq!(
        options.commit_msg,
        Some("Custom commit message".to_string())
    );
}

#[test]
fn test_pr_options_builder_as_draft() {
    let options = PrOptions::new(
        "Draft PR".to_string(),
        "Draft body".to_string(),
        "token".to_string(),
    )
    .as_draft();

    assert_eq!(options.draft, true);
}

#[test]
fn test_pr_options_builder_create_only() {
    let options = PrOptions::new(
        "Local PR".to_string(),
        "Local body".to_string(),
        "token".to_string(),
    )
    .create_only();

    assert_eq!(options.create_only, true);
}

#[test]
fn test_pr_options_builder_chaining() {
    let options = PrOptions::new(
        "Chained PR".to_string(),
        "Chained body".to_string(),
        "token".to_string(),
    )
    .with_branch_name("feature/chain".to_string())
    .with_base_branch("main".to_string())
    .with_commit_message("Chained commit".to_string())
    .as_draft()
    .create_only();

    assert_eq!(options.branch_name, Some("feature/chain".to_string()));
    assert_eq!(options.base_branch, Some("main".to_string()));
    assert_eq!(options.commit_msg, Some("Chained commit".to_string()));
    assert_eq!(options.draft, true);
    assert_eq!(options.create_only, true);
}

#[test]
fn test_github_error_api_error_display() {
    let error = GitHubError::ApiError("Resource not found".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "GitHub API error: Resource not found");
}

#[test]
fn test_github_error_auth_error_display() {
    let error = GitHubError::AuthError;
    let display = format!("{}", error);
    assert_eq!(display, "GitHub authentication error");
}

#[test]
fn test_github_error_network_error_display() {
    let error = GitHubError::NetworkError("Connection timeout".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "Network error: Connection timeout");
}

#[test]
fn test_github_error_parse_error_display() {
    let error = GitHubError::ParseError("Invalid JSON response".to_string());
    let display = format!("{}", error);
    assert_eq!(display, "Parse error: Invalid JSON response");
}

#[test]
fn test_github_error_debug_format() {
    let error = GitHubError::ApiError("Debug test".to_string());
    let debug = format!("{:?}", error);
    assert!(debug.contains("ApiError"));
    assert!(debug.contains("Debug test"));
}

#[test]
fn test_github_error_as_error_trait() {
    let error = GitHubError::NetworkError("Test error".to_string());
    let error_trait: &dyn Error = &error;

    // Test that it implements Error trait
    assert!(error_trait.source().is_none());
    assert_eq!(error_trait.to_string(), "Network error: Test error");
}

#[test]
fn test_github_repo_serialization() {
    let repo = GitHubRepo {
        id: 123456,
        name: "test-repo".to_string(),
        full_name: "owner/test-repo".to_string(),
        html_url: "https://github.com/owner/test-repo".to_string(),
        clone_url: "https://github.com/owner/test-repo.git".to_string(),
        default_branch: "main".to_string(),
    };

    let json = serde_json::to_string(&repo).unwrap();
    assert!(json.contains("\"id\":123456"));
    assert!(json.contains("\"name\":\"test-repo\""));
    assert!(json.contains("\"full_name\":\"owner/test-repo\""));
}

#[test]
fn test_github_repo_deserialization() {
    let json = r#"{
        "id": 789012,
        "name": "example-repo",
        "full_name": "user/example-repo",
        "html_url": "https://github.com/user/example-repo",
        "clone_url": "https://github.com/user/example-repo.git",
        "default_branch": "develop"
    }"#;

    let repo: GitHubRepo = serde_json::from_str(json).unwrap();
    assert_eq!(repo.id, 789012);
    assert_eq!(repo.name, "example-repo");
    assert_eq!(repo.full_name, "user/example-repo");
    assert_eq!(repo.default_branch, "develop");
}

#[test]
fn test_user_serialization() {
    let user = User {
        id: 12345,
        login: "testuser".to_string(),
        html_url: "https://github.com/testuser".to_string(),
    };

    let json = serde_json::to_string(&user).unwrap();
    assert!(json.contains("\"id\":12345"));
    assert!(json.contains("\"login\":\"testuser\""));
    assert!(json.contains("\"html_url\":\"https://github.com/testuser\""));
}

#[test]
fn test_user_deserialization() {
    let json = r#"{
        "id": 67890,
        "login": "example-user",
        "html_url": "https://github.com/example-user"
    }"#;

    let user: User = serde_json::from_str(json).unwrap();
    assert_eq!(user.id, 67890);
    assert_eq!(user.login, "example-user");
    assert_eq!(user.html_url, "https://github.com/example-user");
}

#[test]
fn test_pull_request_serialization() {
    let user = User {
        id: 1,
        login: "author".to_string(),
        html_url: "https://github.com/author".to_string(),
    };

    let pr = PullRequest {
        id: 111,
        number: 42,
        title: "Test PR".to_string(),
        body: Some("Test body".to_string()),
        html_url: "https://github.com/owner/repo/pull/42".to_string(),
        state: "open".to_string(),
        user,
    };

    let json = serde_json::to_string(&pr).unwrap();
    assert!(json.contains("\"id\":111"));
    assert!(json.contains("\"number\":42"));
    assert!(json.contains("\"title\":\"Test PR\""));
    assert!(json.contains("\"state\":\"open\""));
}

#[test]
fn test_pull_request_deserialization() {
    let json = r#"{
        "id": 222,
        "number": 84,
        "title": "Example PR",
        "body": "Example body",
        "html_url": "https://github.com/owner/repo/pull/84",
        "state": "closed",
        "user": {
            "id": 2,
            "login": "contributor",
            "html_url": "https://github.com/contributor"
        }
    }"#;

    let pr: PullRequest = serde_json::from_str(json).unwrap();
    assert_eq!(pr.id, 222);
    assert_eq!(pr.number, 84);
    assert_eq!(pr.title, "Example PR");
    assert_eq!(pr.body, Some("Example body".to_string()));
    assert_eq!(pr.state, "closed");
    assert_eq!(pr.user.login, "contributor");
}

#[test]
fn test_pull_request_with_none_body() {
    let json = r#"{
        "id": 333,
        "number": 126,
        "title": "No Body PR",
        "body": null,
        "html_url": "https://github.com/owner/repo/pull/126",
        "state": "draft",
        "user": {
            "id": 3,
            "login": "reviewer",
            "html_url": "https://github.com/reviewer"
        }
    }"#;

    let pr: PullRequest = serde_json::from_str(json).unwrap();
    assert_eq!(pr.body, None);
    assert_eq!(pr.state, "draft");
}

#[test]
fn test_pr_options_empty_strings() {
    let options = PrOptions::new("".to_string(), "".to_string(), "".to_string());

    assert_eq!(options.title, "");
    assert_eq!(options.body, "");
    assert_eq!(options.token, "");
}

#[test]
fn test_pr_options_unicode_content() {
    let options = PrOptions::new(
        "测试PR".to_string(),
        "测试内容 with émojis 🚀".to_string(),
        "token".to_string(),
    );

    assert_eq!(options.title, "测试PR");
    assert!(options.body.contains("🚀"));
}

#[test]
fn test_github_error_variants_coverage() {
    // Test all error variants to ensure coverage
    let api_error = GitHubError::ApiError("API issue".to_string());
    let auth_error = GitHubError::AuthError;
    let network_error = GitHubError::NetworkError("Network issue".to_string());
    let parse_error = GitHubError::ParseError("Parse issue".to_string());

    // Test that all variants can be formatted
    assert!(format!("{}", api_error).contains("API issue"));
    assert_eq!(format!("{}", auth_error), "GitHub authentication error");
    assert!(format!("{}", network_error).contains("Network issue"));
    assert!(format!("{}", parse_error).contains("Parse issue"));
}

#[test]
fn test_pull_request_params_with_special_characters() {
    let params = PullRequestParams::new(
        "owner-with-dash",
        "repo_with_underscore",
        "Title with spaces & symbols!",
        "Body with\nnewlines\tand\ttabs",
        "feature/branch-name",
        "main/branch",
        false,
    );

    assert!(params.owner.contains("-"));
    assert!(params.repo.contains("_"));
    assert!(params.title.contains("&"));
    assert!(params.body.contains("\n"));
    assert!(params.head.contains("/"));
}

#[test]
fn test_constants_module_access() {
    use repos::github::types::constants::{DEFAULT_USER_AGENT, GITHUB_API_BASE};

    // Test that constants are accessible
    assert!(GITHUB_API_BASE.contains("api.github.com"));
    assert!(DEFAULT_USER_AGENT.contains("repos"));
}
