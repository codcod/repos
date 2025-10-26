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

    #[test]
    fn test_pull_request_module_exists() {
        // This test ensures the module compiles and can be imported
        let client = GitHubClient::new(None);
        assert!(client.auth.is_none());
    }
}
