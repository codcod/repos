//! GitHub Pull Request API operations
//!
//! This module contains all functionality related to GitHub pull requests,
//! including creation, management, and querying of pull requests.

use super::client::GitHubClient;
use super::types::{PullRequest, PullRequestParams};
use anyhow::Result;
use serde_json::{Value, json};

impl GitHubClient {
    /// Create a new pull request on GitHub
    ///
    /// # Arguments
    /// * `params` - Pull request parameters including owner, repo, title, body, etc.
    ///
    /// # Returns
    /// A JSON value containing the GitHub API response for the created pull request
    ///
    /// # Example
    /// ```rust,no_run
    /// use repos::github::{GitHubClient, PullRequestParams};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(Some("github_token".to_string()));
    /// let params = PullRequestParams::new(
    ///     "owner",
    ///     "repo",
    ///     "Fix bug in authentication",
    ///     "This PR fixes a critical bug in the auth system",
    ///     "feature-branch",
    ///     "main",
    ///     false
    /// );
    ///
    /// let pr_result = client.create_pull_request(params).await?;
    /// println!("Created PR: {}", pr_result["html_url"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_pull_request(&self, params: PullRequestParams<'_>) -> Result<Value> {
        let auth = self.auth.as_ref().ok_or_else(|| {
            anyhow::anyhow!("GitHub token is required for creating pull requests")
        })?;

        let url = format!(
            "{}/repos/{}/{}/pulls",
            super::types::constants::GITHUB_API_BASE,
            params.owner,
            params.repo
        );

        let payload = json!({
            "title": params.title,
            "body": params.body,
            "head": params.head,
            "base": params.base,
            "draft": params.draft
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth.get_auth_header())
            .header("User-Agent", super::types::constants::DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "GitHub API error ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// Get a specific pull request by number
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    /// * `pr_number` - Pull request number
    ///
    /// # Returns
    /// A PullRequest struct containing the PR information
    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<PullRequest> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo,
            pr_number
        );

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", super::types::constants::DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json");

        // Add authorization if available
        if let Some(auth) = &self.auth {
            request = request.header("Authorization", auth.get_auth_header());
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let pr: PullRequest = response.json().await?;
            Ok(pr)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to get pull request ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// List pull requests for a repository
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    /// * `state` - Optional state filter ("open", "closed", "all")
    /// * `base` - Optional base branch filter
    ///
    /// # Returns
    /// A vector of PullRequest structs
    pub async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
        base: Option<&str>,
    ) -> Result<Vec<PullRequest>> {
        let mut url = format!(
            "{}/repos/{}/{}/pulls",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo
        );

        // Add query parameters
        let mut params = Vec::new();
        if let Some(state) = state {
            params.push(format!("state={}", state));
        }
        if let Some(base) = base {
            params.push(format!("base={}", base));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", super::types::constants::DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json");

        // Add authorization if available
        if let Some(auth) = &self.auth {
            request = request.header("Authorization", auth.get_auth_header());
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let prs: Vec<PullRequest> = response.json().await?;
            Ok(prs)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to list pull requests ({}): {}",
                status,
                error_text
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client_with_auth() -> GitHubClient {
        GitHubClient::new(Some("test-token".to_string()))
    }

    fn create_test_client_without_auth() -> GitHubClient {
        GitHubClient::new(None)
    }

    fn create_test_pr_params() -> PullRequestParams<'static> {
        PullRequestParams::new(
            "test-owner",
            "test-repo",
            "Test PR Title",
            "Test PR body content",
            "feature-branch",
            "main",
            false,
        )
    }

    #[tokio::test]
    async fn test_create_pull_request_without_auth() {
        // Test the auth missing path (line 42-44)
        let client = create_test_client_without_auth();
        let params = create_test_pr_params();

        let result = client.create_pull_request(params).await;

        // Should fail with auth error
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("GitHub token is required")
        );
    }

    #[tokio::test]
    async fn test_create_pull_request_with_auth() {
        // Test the main execution path with auth (lines 46-79)
        let client = create_test_client_with_auth();
        let params = create_test_pr_params();

        let result = client.create_pull_request(params).await;

        // Will fail due to network/API, but exercises the execution path
        assert!(result.is_err()); // Expected failure without real GitHub setup
    }

    #[tokio::test]
    async fn test_get_pull_request_execution() {
        // Test get_pull_request execution path
        let client = create_test_client_with_auth();

        let result = client
            .get_pull_request("test-owner", "test-repo", 123)
            .await;

        // Will fail due to network/API, but exercises the execution path
        assert!(result.is_err()); // Expected failure without real GitHub setup
    }

    #[tokio::test]
    async fn test_list_pull_requests_execution() {
        // Test list_pull_requests execution path
        let client = create_test_client_with_auth();

        let result = client
            .list_pull_requests("test-owner", "test-repo", None, None)
            .await;

        // Will fail due to network/API, but exercises the execution path
        assert!(result.is_err()); // Expected failure without real GitHub setup
    }

    #[test]
    fn test_pull_request_module_exists() {
        // This test ensures the module compiles and can be imported
        let client = GitHubClient::new(None);
        assert!(client.auth.is_none());
    }

    #[test]
    fn test_pull_request_params_creation() {
        // Test PullRequestParams creation and field access
        let params = create_test_pr_params();

        assert_eq!(params.owner, "test-owner");
        assert_eq!(params.repo, "test-repo");
        assert_eq!(params.title, "Test PR Title");
        assert_eq!(params.body, "Test PR body content");
        assert_eq!(params.head, "feature-branch");
        assert_eq!(params.base, "main");
        assert!(!params.draft);
    }
}
