//! GitHub API integration module
//!
//! This module provides a comprehensive interface for interacting with GitHub's REST API.
//! It follows a modular design where different API endpoints are organized into separate
//! sub-modules for better maintainability and organization.
//!
//! ## Architecture
//!
//! - [`client`]: Core GitHub client with authentication and URL parsing
//! - [`auth`]: Authentication handling and token management
//! - [`pull_requests`]: Pull request creation and management
//! - [`repositories`]: Repository information and releases
//! - [`types`]: Data structures and type definitions
//! - [`api`]: High-level workflow functions
//!
//! ## Features
//!
//! - **Modular Design**: API operations grouped by functionality
//! - **Authentication**: Secure token-based authentication
//! - **Error Handling**: Comprehensive error types and handling
//! - **Enterprise Support**: Works with both GitHub.com and GitHub Enterprise
//! - **Async/Await**: Fully async API with tokio support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use repos::github::{GitHubClient, PrOptions};
//! use repos::config::Repository;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a client
//! let client = GitHubClient::new(Some("your_token".to_string()));
//!
//! // Parse a GitHub URL
//! let (owner, repo) = client.parse_github_url("https://github.com/rust-lang/rust")?;
//!
//! // Get repository information
//! let repo_info = client.get_repository(&owner, &repo).await?;
//! println!("Repository: {}", repo_info.full_name);
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod auth;
pub mod client;
pub mod pull_requests;
pub mod repositories;
pub mod types;

// Re-export commonly used items for convenience
pub use api::create_pr_from_workspace;
pub use auth::GitHubAuth;
pub use client::GitHubClient;
pub use types::{GitHubRepo, PrOptions, PullRequest, PullRequestParams, User};

// Re-export constants for easy access
pub use crate::constants::github::{DEFAULT_BRANCH_PREFIX, DEFAULT_USER_AGENT};
