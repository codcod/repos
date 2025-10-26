//! GitHub API client implementation
//!
//! This module provides the main `GitHubClient` struct which serves as the entry point
//! for all GitHub API operations. The client encapsulates authentication and HTTP client
//! state, making it easy to perform various GitHub operations.
//!
//! ## Architecture
//!
//! The `GitHubClient` follows a modular design where different API endpoints are organized
//! into separate modules:
//! - `pull_requests.rs` - Pull request operations
//! - `repositories.rs` - Repository information and releases
//!
//! Each module extends the `GitHubClient` with `impl` blocks containing related methods.

use super::auth::GitHubAuth;
use anyhow::Result;
use reqwest::Client;

/// GitHub API client for interacting with GitHub's REST API
///
/// This client provides a unified interface for GitHub API operations, managing
/// authentication and HTTP client state. Different API endpoints are organized
/// into separate modules that extend this client with specific functionality.
///
/// ## Features
///
/// - **Authentication Management**: Handles GitHub token authentication
/// - **URL Parsing**: Supports both GitHub.com and GitHub Enterprise URLs
/// - **Modular Design**: API operations are organized by functionality
/// - **Error Handling**: Comprehensive error handling for API responses
///
/// ## Example
///
/// ```rust,no_run
/// use repos::github::GitHubClient;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Create client with authentication
/// let client = GitHubClient::new(Some("your_github_token".to_string()));
///
/// // Parse repository URL
/// let (owner, repo) = client.parse_github_url("https://github.com/owner/repo")?;
///
/// // Use client for various operations (see specific modules for examples)
/// // - Pull requests: client.create_pull_request()
/// // - Repositories: client.get_repository()
/// # Ok(())
/// # }
/// ```
pub struct GitHubClient {
    pub(crate) client: Client,
    pub(crate) auth: Option<GitHubAuth>,
}

impl GitHubClient {
    /// Create a new GitHub client
    ///
    /// # Arguments
    /// * `token` - Optional GitHub personal access token for authentication
    ///
    /// # Returns
    /// A new GitHubClient instance
    ///
    /// # Example
    /// ```rust
    /// use repos::github::GitHubClient;
    ///
    /// // Client without authentication (for public repositories)
    /// let public_client = GitHubClient::new(None);
    ///
    /// // Client with authentication (for private repos and higher rate limits)
    /// let auth_client = GitHubClient::new(Some("your_token".to_string()));
    /// ```
    pub fn new(token: Option<String>) -> Self {
        let auth = token.map(GitHubAuth::new);
        Self {
            client: Client::new(),
            auth,
        }
    }

    /// Parse GitHub URL to extract owner and repository name
    ///
    /// Supports both github.com and enterprise GitHub instances with various URL formats:
    /// - SSH: `git@github.com:owner/repo` or `git@github-enterprise:owner/repo`
    /// - HTTPS: `https://github.com/owner/repo` or `https://github-enterprise/owner/repo`
    /// - Legacy: `github.com/owner/repo`
    ///
    /// # Arguments
    /// * `url` - The GitHub repository URL to parse
    ///
    /// # Returns
    /// A tuple containing (owner, repository_name)
    ///
    /// # Errors
    /// Returns an error if the URL format is not recognized as a valid GitHub URL
    ///
    /// # Example
    /// ```rust
    /// use repos::github::GitHubClient;
    ///
    /// let client = GitHubClient::new(None);
    /// let (owner, repo) = client.parse_github_url("https://github.com/rust-lang/rust").unwrap();
    /// assert_eq!(owner, "rust-lang");
    /// assert_eq!(repo, "rust");
    /// ```
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

        Err(anyhow::anyhow!("Invalid GitHub URL format: {}", url))
    }

    /// Check if the client has authentication configured
    ///
    /// # Returns
    /// `true` if the client has a GitHub token configured, `false` otherwise
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_some()
    }

    /// Get the authentication token (if available)
    ///
    /// # Returns
    /// `Some(token)` if authenticated, `None` otherwise
    pub fn token(&self) -> Option<&str> {
        self.auth.as_ref().map(|auth| auth.token())
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
