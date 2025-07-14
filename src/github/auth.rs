//! GitHub authentication utilities

use anyhow::Result;

pub struct GitHubAuth {
    token: String,
}

impl GitHubAuth {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub fn validate_token(&self) -> Result<()> {
        if self.token.is_empty() {
            anyhow::bail!("GitHub token is required");
        }
        Ok(())
    }
}
