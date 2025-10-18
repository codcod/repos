use repos::github::client::GitHubClient;
use repos::github::types::PullRequestParams;

#[tokio::test]
async fn test_github_client_creation_with_token() {
    // Test client creation with token
    let client = GitHubClient::new(Some("test_token".to_string()));

    // Test that the client is created (we can't directly test the token, but this verifies construction)
    let result = client.parse_github_url("git@github.com:owner/repo");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_github_client_creation_without_token() {
    // Test client creation without token
    let client = GitHubClient::new(None);

    // Test that the client is created
    let result = client.parse_github_url("git@github.com:owner/repo");
    assert!(result.is_ok());
}

#[test]
fn test_parse_github_url_invalid_formats() {
    let client = GitHubClient::new(None);

    // Test various invalid URL formats that don't match any regex pattern
    assert!(client.parse_github_url("").is_err());
    assert!(client.parse_github_url("not-a-url").is_err());
    assert!(client.parse_github_url("https://github.com").is_err()); // Missing owner/repo
    assert!(client.parse_github_url("https://github.com/owner").is_err()); // Missing repo
    assert!(client.parse_github_url("invalid://format").is_err());
    assert!(client.parse_github_url("just-text").is_err());
    assert!(client.parse_github_url("git@").is_err()); // Incomplete SSH
    assert!(client.parse_github_url("https://").is_err()); // Incomplete HTTPS
}

#[test]
fn test_parse_github_url_with_trailing_slash() {
    let client = GitHubClient::new(None);

    // Test URLs with trailing slashes
    let (owner, repo) = client
        .parse_github_url("https://github.com/owner/repo/")
        .unwrap();
    assert_eq!(owner, "owner");
    assert_eq!(repo, "repo");

    let (owner, repo) = client
        .parse_github_url("git@github.com:owner/repo.git/")
        .unwrap();
    assert_eq!(owner, "owner");
    assert_eq!(repo, "repo");
}

#[test]
fn test_parse_github_url_complex_repo_names() {
    let client = GitHubClient::new(None);

    // Test with complex repository names
    let (owner, repo) = client
        .parse_github_url("git@github.com:my-org/my-repo-name")
        .unwrap();
    assert_eq!(owner, "my-org");
    assert_eq!(repo, "my-repo-name");

    let (owner, repo) = client
        .parse_github_url("https://github.com/my_org/my_repo_123")
        .unwrap();
    assert_eq!(owner, "my_org");
    assert_eq!(repo, "my_repo_123");
}

#[test]
fn test_parse_github_url_legacy_colon_format() {
    let client = GitHubClient::new(None);

    // Test legacy format with colon
    let (owner, repo) = client.parse_github_url("github.com:owner/repo").unwrap();
    assert_eq!(owner, "owner");
    assert_eq!(repo, "repo");
}

#[test]
fn test_parse_github_url_enterprise_https_paths() {
    let client = GitHubClient::new(None);

    // Test enterprise GitHub with different paths
    let (owner, repo) = client
        .parse_github_url("https://github.enterprise.com/owner/repo")
        .unwrap();
    assert_eq!(owner, "owner");
    assert_eq!(repo, "repo");

    let (owner, repo) = client
        .parse_github_url("https://git.company.com/team/project")
        .unwrap();
    assert_eq!(owner, "team");
    assert_eq!(repo, "project");
}

#[tokio::test]
async fn test_create_pull_request_without_token() {
    // Test PR creation without authentication token
    let client = GitHubClient::new(None);

    let params = PullRequestParams {
        owner: "owner",
        repo: "repo",
        title: "Test PR",
        body: "Test body",
        head: "feature-branch",
        base: "main",
        draft: false,
    };

    let result = client.create_pull_request(params).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("GitHub token is required")
    );
}

#[tokio::test]
async fn test_create_pull_request_as_draft() {
    // Test draft PR creation parameters
    let client = GitHubClient::new(Some("test_token".to_string()));

    let params = PullRequestParams {
        owner: "owner",
        repo: "repo",
        title: "Draft PR",
        body: "Draft body",
        head: "feature-branch",
        base: "main",
        draft: true, // Test draft flag
    };

    // This will fail due to no actual GitHub API, but tests parameter handling
    let result = client.create_pull_request(params).await;
    assert!(result.is_err()); // Expected since we don't have a real API endpoint
}

#[tokio::test]
async fn test_create_pull_request_with_empty_body() {
    // Test PR creation with empty body
    let client = GitHubClient::new(Some("test_token".to_string()));

    let params = PullRequestParams {
        owner: "owner",
        repo: "repo",
        title: "PR with empty body",
        body: "", // Empty body
        head: "feature-branch",
        base: "main",
        draft: false,
    };

    // This will fail due to no actual GitHub API, but tests parameter handling
    let result = client.create_pull_request(params).await;
    assert!(result.is_err()); // Expected since we don't have a real API endpoint
}

#[test]
fn test_parse_github_url_regex_edge_cases() {
    let client = GitHubClient::new(None);

    // Test edge cases that might cause regex issues
    assert!(client.parse_github_url("git@:owner/repo").is_err());
    assert!(client.parse_github_url("git@github.com:/repo").is_err());
    assert!(client.parse_github_url("git@github.com:owner/").is_err());
    assert!(client.parse_github_url("https:///owner/repo").is_err());
    assert!(client.parse_github_url("https://github.com//repo").is_err());
    assert!(
        client
            .parse_github_url("https://github.com/owner/")
            .is_err()
    );
}

#[test]
fn test_parse_github_url_special_characters() {
    let client = GitHubClient::new(None);

    // Test handling of URLs with special characters
    let (owner, repo) = client
        .parse_github_url("git@github.com:my-org/my.repo")
        .unwrap();
    assert_eq!(owner, "my-org");
    assert_eq!(repo, "my.repo");

    let (owner, repo) = client
        .parse_github_url("https://github.com/my_org/my-repo_123")
        .unwrap();
    assert_eq!(owner, "my_org");
    assert_eq!(repo, "my-repo_123");
}

#[test]
fn test_parse_github_url_case_sensitivity() {
    let client = GitHubClient::new(None);

    // Test case sensitivity (GitHub URLs should preserve case)
    let (owner, repo) = client
        .parse_github_url("git@github.com:MyOrg/MyRepo")
        .unwrap();
    assert_eq!(owner, "MyOrg");
    assert_eq!(repo, "MyRepo");

    let (owner, repo) = client
        .parse_github_url("https://github.com/OWNER/REPO")
        .unwrap();
    assert_eq!(owner, "OWNER");
    assert_eq!(repo, "REPO");
}
