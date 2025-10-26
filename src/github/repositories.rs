//! GitHub Repository API operations
//!
//! This module contains functionality for interacting with GitHub repositories,
//! including getting repository information, releases, and other repo-level operations.

use super::client::GitHubClient;
use super::types::GitHubRepo;
use anyhow::Result;
use serde_json::Value;

impl GitHubClient {
    /// Get repository information from GitHub
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// A GitHubRepo struct containing repository information
    ///
    /// # Example
    /// ```rust,no_run
    /// use repos::github::GitHubClient;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(Some("github_token".to_string()));
    /// let repo_info = client.get_repository("octocat", "Hello-World").await?;
    /// println!("Repository: {}", repo_info.full_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let url = format!(
            "{}/repos/{}/{}",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo
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
            let repo_info: GitHubRepo = response.json().await?;
            Ok(repo_info)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to get repository information ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// Get the latest release of a repository
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// A JSON Value containing the latest release information
    ///
    /// # Example
    /// ```rust,no_run
    /// use repos::github::GitHubClient;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(None); // Public API, no token needed
    /// let latest_release = client.get_latest_release("rust-lang", "rust").await?;
    /// println!("Latest release: {}", latest_release["tag_name"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_release(&self, owner: &str, repo: &str) -> Result<Value> {
        let url = format!(
            "{}/repos/{}/{}/releases/latest",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo
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
            let release: Value = response.json().await?;
            Ok(release)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to get latest release ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// List all releases for a repository
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    /// * `per_page` - Optional number of results per page (default: 30, max: 100)
    /// * `page` - Optional page number for pagination (default: 1)
    ///
    /// # Returns
    /// A vector of JSON Values containing release information
    pub async fn list_releases(
        &self,
        owner: &str,
        repo: &str,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<Vec<Value>> {
        let mut url = format!(
            "{}/repos/{}/{}/releases",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo
        );

        // Add query parameters
        let mut params = Vec::new();
        if let Some(per_page) = per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = page {
            params.push(format!("page={}", page));
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
            let releases: Vec<Value> = response.json().await?;
            Ok(releases)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to list releases ({}): {}",
                status,
                error_text
            ))
        }
    }

    /// Get repository topics (tags/labels)
    ///
    /// # Arguments
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    ///
    /// # Returns
    /// A vector of topic strings
    pub async fn get_repository_topics(&self, owner: &str, repo: &str) -> Result<Vec<String>> {
        let url = format!(
            "{}/repos/{}/{}/topics",
            super::types::constants::GITHUB_API_BASE,
            owner,
            repo
        );

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", super::types::constants::DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.mercy-preview+json"); // Topics API requires this accept header

        // Add authorization if available
        if let Some(auth) = &self.auth {
            request = request.header("Authorization", auth.get_auth_header());
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            let topics = result["names"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Ok(topics)
        } else {
            let status = response.status();
            let error_text = response.text().await?;
            Err(anyhow::anyhow!(
                "Failed to get repository topics ({}): {}",
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
    fn test_repository_module_exists() {
        // This test ensures the module compiles and can be imported
        let client = GitHubClient::new(None);
        assert!(client.auth.is_none());
    }
}
