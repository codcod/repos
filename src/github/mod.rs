//! GitHub API integration module

pub mod api;
pub mod auth;
pub mod client;
pub mod types;

// Re-export commonly used items for convenience
pub use api::create_pull_request;
pub use auth::GitHubAuth;
pub use client::GitHubClient;
pub use types::{PrOptions, PullRequestParams};
