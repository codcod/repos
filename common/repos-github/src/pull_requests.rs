//! Pull request operations

use crate::client::GitHubClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct CreatePullRequestPayload<'a> {
    title: &'a str,
    head: &'a str,
    base: &'a str,
    body: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    draft: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct PullRequest {
    pub html_url: String,
    pub number: u64,
    pub id: u64,
    pub title: String,
    pub state: String,
}

/// Parameters for creating a pull request
#[derive(Debug, Clone)]
pub struct PullRequestParams<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub title: &'a str,
    pub head: &'a str,
    pub base: &'a str,
    pub body: &'a str,
    pub draft: bool,
}

impl<'a> PullRequestParams<'a> {
    pub fn new(
        owner: &'a str,
        repo: &'a str,
        title: &'a str,
        head: &'a str,
        base: &'a str,
        body: &'a str,
        draft: bool,
    ) -> Self {
        Self {
            owner,
            repo,
            title,
            head,
            base,
            body,
            draft,
        }
    }
}

impl GitHubClient {
    /// Create a pull request on GitHub
    ///
    /// # Arguments
    /// * `params` - Pull request parameters including owner, repo, title, head, base, body, and draft status
    ///
    /// # Returns
    /// A PullRequest struct containing the created PR information
    ///
    /// # Errors
    /// Returns an error if:
    /// - No authentication token is configured
    /// - The API request fails
    /// - The response cannot be parsed
    pub async fn create_pull_request(&self, params: PullRequestParams<'_>) -> Result<PullRequest> {
        if self.token.is_none() {
            anyhow::bail!(
                "GitHub token is required for creating pull requests. Set GITHUB_TOKEN environment variable."
            );
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            params.owner, params.repo
        );

        let payload = CreatePullRequestPayload {
            title: params.title,
            head: params.head,
            base: params.base,
            body: params.body,
            draft: if params.draft { Some(true) } else { None },
        };

        let mut request = self.client.post(&url).header("User-Agent", "repos-cli");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "Failed to create pull request ({} {}): {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_text
            ));
        }

        let pr: PullRequest = response
            .json()
            .await
            .context("Failed to parse PR creation response")?;
        Ok(pr)
    }
}
