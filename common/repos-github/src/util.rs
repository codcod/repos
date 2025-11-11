//! Utility functions for GitHub operations

use anyhow::{Result, anyhow};

/// Parse GitHub URL to extract owner and repository name
///
/// Supports various GitHub URL formats:
/// - SSH: `git@github.com:owner/repo.git`
/// - HTTPS: `https://github.com/owner/repo.git`
/// - Legacy: `github.com/owner/repo`
///
/// # Arguments
/// * `url` - The GitHub repository URL to parse
///
/// # Returns
/// A tuple containing (owner, repository_name)
///
/// # Errors
/// Returns an error if the URL format is not recognized
pub fn parse_github_url(url: &str) -> Result<(String, String)> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    // Handle SSH URLs: git@github.com:owner/repo or git@github-enterprise:owner/repo
    if url.starts_with("git@")
        && let Some(colon_pos) = url.find(':')
    {
        let after_colon = &url[colon_pos + 1..];
        let parts: Vec<&str> = after_colon.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Handle HTTPS URLs: https://github.com/owner/repo or https://github-enterprise/owner/repo
    if url.starts_with("https://") || url.starts_with("http://") {
        let without_protocol = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        let parts: Vec<&str> = without_protocol.split('/').collect();
        if parts.len() >= 3 {
            return Ok((parts[1].to_string(), parts[2].to_string()));
        }
    }

    // Legacy support: github.com/owner/repo
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let idx = parts.len() - 2;
            return Ok((parts[idx].to_string(), parts[idx + 1].to_string()));
        }
    }

    Err(anyhow!("Invalid GitHub URL format: {}", url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let (owner, repo) = parse_github_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_https_url() {
        let (owner, repo) = parse_github_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_legacy_url() {
        let (owner, repo) = parse_github_url("github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_github_url("invalid-url").is_err());
    }
}
