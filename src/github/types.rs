//! GitHub API types and data structures

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Parameters for creating a pull request
#[derive(Debug, Clone)]
pub struct PullRequestParams<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub head: &'a str,
    pub base: &'a str,
    pub draft: bool,
}

impl<'a> PullRequestParams<'a> {
    pub fn new(
        owner: &'a str,
        repo: &'a str,
        title: &'a str,
        body: &'a str,
        head: &'a str,
        base: &'a str,
        draft: bool,
    ) -> Self {
        Self {
            owner,
            repo,
            title,
            body,
            head,
            base,
            draft,
        }
    }
}

/// Pull request options for creation
#[derive(Debug, Clone)]
pub struct PrOptions {
    pub title: String,
    pub body: String,
    pub branch_name: Option<String>,
    pub base_branch: Option<String>,
    pub commit_msg: Option<String>,
    pub draft: bool,
    pub token: String,
    pub create_only: bool,
}

impl PrOptions {
    pub fn new(title: String, body: String, token: String) -> Self {
        Self {
            title,
            body,
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token,
            create_only: false,
        }
    }

    pub fn with_branch_name(mut self, branch_name: String) -> Self {
        self.branch_name = Some(branch_name);
        self
    }

    pub fn with_base_branch(mut self, base_branch: String) -> Self {
        self.base_branch = Some(base_branch);
        self
    }

    pub fn with_commit_message(mut self, commit_msg: String) -> Self {
        self.commit_msg = Some(commit_msg);
        self
    }

    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }

    pub fn create_only(mut self) -> Self {
        self.create_only = true;
        self
    }
}

/// GitHub API error types
#[derive(Debug)]
pub enum GitHubError {
    ApiError(String),
    AuthError,
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for GitHubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitHubError::ApiError(msg) => write!(f, "GitHub API error: {msg}"),
            GitHubError::AuthError => write!(f, "GitHub authentication error"),
            GitHubError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            GitHubError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl Error for GitHubError {}

/// GitHub repository information
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
}

/// GitHub user information
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub html_url: String,
}

/// Pull request response from GitHub API
#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
    pub state: String,
    pub user: User,
}

/// Constants for GitHub API
pub mod constants {
    pub const GITHUB_API_BASE: &str = "https://api.github.com";
    pub const DEFAULT_USER_AGENT: &str = "repos/0.1.0";
}
