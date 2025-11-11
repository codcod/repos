# repos-github

A shared library for GitHub API interactions used by the `repos` CLI tool and its plugins.

## Purpose

This library centralizes all GitHub API communication logic, providing a consistent and reusable interface for:

- Repository information retrieval
- Topic fetching
- Authentication handling
- Error management

## Usage

```rust
use repos_github::{GitHubClient, PullRequestParams};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new GitHub client (automatically reads GITHUB_TOKEN from env)
    let client = GitHubClient::new(None);

    // Get repository details including topics
    let repo_details = client.get_repository_details("owner", "repo-name").await?;
    println!("Topics: {:?}", repo_details.topics);

    // Create a pull request
    let pr_params = PullRequestParams::new(
        "owner",
        "repo-name",
        "My Test PR",
        "feature-branch",
        "main",
        "This is the body of the PR.",
        true, // draft
    );
    let pr = client.create_pull_request(pr_params).await?;
    println!("Created PR: {}", pr.html_url);

    Ok(())
}
```

## Authentication

The library automatically reads the `GITHUB_TOKEN` environment variable for authentication. This is required for:

- Private repositories
- Avoiding API rate limits
- Accessing organization repositories

## Error Handling

The library provides detailed error messages for common scenarios:

- **403 Forbidden**: Indicates missing or insufficient permissions
- **404 Not Found**: Repository doesn't exist or isn't accessible
- Network errors and timeouts

## Integration

This library is used by:

- `repos-validate` plugin: For connectivity checks and topic supplementation
- Future plugins that need GitHub API access

## Benefits

- **DRY Principle**: Single source of truth for GitHub API logic
- **Consistency**: All components use the same authentication and error handling
- **Maintainability**: Changes to GitHub API interactions are made in one place
- **Testability**: Centralized logic is easier to unit test
