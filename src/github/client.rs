//! GitHub API client implementation

use super::auth::GitHubAuth;
use super::types::{PullRequestParams, constants::*};
use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};

/// GitHub API client
pub struct GitHubClient {
    client: Client,
    auth: Option<GitHubAuth>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(token: Option<String>) -> Self {
        let auth = token.map(GitHubAuth::new);
        Self {
            client: Client::new(),
            auth,
        }
    }

    /// Parse GitHub URL to extract owner and repository name
    /// Supports both github.com and enterprise GitHub instances
    pub fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches('/').trim_end_matches(".git");

        // Handle SSH URLs: git@github.com:owner/repo or git@github-enterprise:owner/repo
        if let Some(captures) = regex::Regex::new(r"git@([^:]+):([^/]+)/(.+)")?.captures(url) {
            let owner = captures.get(2).unwrap().as_str().to_string();
            let repo = captures.get(3).unwrap().as_str().to_string();
            return Ok((owner, repo));
        }

        // Handle HTTPS URLs: https://github.com/owner/repo or https://github-enterprise/owner/repo
        if let Some(captures) = regex::Regex::new(r"https://([^/]+)/([^/]+)/(.+)")?.captures(url) {
            let owner = captures.get(2).unwrap().as_str().to_string();
            let repo = captures.get(3).unwrap().as_str().to_string();
            return Ok((owner, repo));
        }

        // Legacy support for github.com URLs with [:/] pattern
        if let Some(captures) = regex::Regex::new(r"github\.com[:/]([^/]+)/([^/]+)")?.captures(url)
        {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            return Ok((owner, repo));
        }

        Err(anyhow::anyhow!("Invalid GitHub URL: {}", url))
    }

    /// Create a pull request
    pub async fn create_pull_request(&self, params: PullRequestParams<'_>) -> Result<Value> {
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub token is required"))?;

        let url = format!(
            "{}/repos/{}/{}/pulls",
            GITHUB_API_BASE, params.owner, params.repo
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
            .header("Authorization", format!("token {}", auth.token()))
            .header("User-Agent", DEFAULT_USER_AGENT)
            .header("Accept", "application/vnd.github.v3+json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            Ok(result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("GitHub API error: {}", error_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_ssh_github_com() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client
            .parse_github_url("git@github.com:owner/repo")
            .unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_ssh_enterprise() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client
            .parse_github_url("git@github-enterprise:nicos_backbase/journey")
            .unwrap();
        assert_eq!(owner, "nicos_backbase");
        assert_eq!(repo, "journey");
    }

    #[test]
    fn test_parse_github_url_https_github_com() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client
            .parse_github_url("https://github.com/owner/repo")
            .unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_https_enterprise() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client
            .parse_github_url("https://github-enterprise/owner/repo")
            .unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_with_git_suffix() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client
            .parse_github_url("git@github-enterprise:owner/repo.git")
            .unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_url_legacy_format() {
        let client = GitHubClient::new(None);
        let (owner, repo) = client.parse_github_url("github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }
}
