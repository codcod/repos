//! Repository-related operations

use crate::client::GitHubClient;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubRepo {
    pub topics: Vec<String>,
}

impl GitHubClient {
    pub async fn get_repository_details(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let mut request = self.client.get(&url).header("User-Agent", "repos-cli");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = if status.as_u16() == 403 {
                if self.token.is_none() {
                    "Access forbidden. This may be a private repository. Set GITHUB_TOKEN environment variable."
                } else {
                    "Access forbidden. Check your GITHUB_TOKEN permissions or repository access."
                }
            } else {
                status.canonical_reason().unwrap_or("Unknown error")
            };
            return Err(anyhow!(
                "Failed to connect ({} {})",
                status.as_u16(),
                error_msg
            ));
        }

        let repo_data: GitHubRepo = response
            .json()
            .await
            .context("Failed to parse GitHub API response")?;
        Ok(repo_data)
    }
}
