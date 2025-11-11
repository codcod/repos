//! GitHub client implementation

/// GitHub API client for making authenticated requests
pub struct GitHubClient {
    pub(crate) client: reqwest::Client,
    pub(crate) token: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client with an optional token
    /// If no token is provided, will try to read from GITHUB_TOKEN environment variable
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: token.or_else(|| std::env::var("GITHUB_TOKEN").ok()),
        }
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new(None)
    }
}
