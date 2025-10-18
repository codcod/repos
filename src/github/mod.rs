//! GitHub API integration module

pub mod api;
pub mod auth;
pub mod client;
pub mod types;

// Re-export commonly used items for convenience
pub use api::create_pr_from_workspace;
pub use auth::GitHubAuth;
pub use client::GitHubClient;
pub use types::{PrOptions, PullRequestParams};

// Re-export constants for easy access
pub use crate::constants::github::{DEFAULT_BRANCH_PREFIX, DEFAULT_USER_AGENT};
