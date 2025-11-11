//! GitHub API client library
//!
//! This library provides a centralized interface for GitHub API operations
//! including repository management, pull request creation, and authentication.
//!
//! ## Modules
//!
//! - [`client`]: Core GitHub client implementation
//! - [`pull_requests`]: Pull request creation and management
//! - [`repositories`]: Repository information retrieval
//! - [`util`]: Utility functions for GitHub operations

mod client;
mod pull_requests;
mod repositories;
mod util;

// Re-export public API
pub use client::GitHubClient;
pub use pull_requests::{PullRequest, PullRequestParams};
pub use repositories::GitHubRepo;
pub use util::parse_github_url;
