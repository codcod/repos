//! GitHub workflow module
//!
//! This module provides high-level workflow functions for GitHub operations
//! using the shared repos-github library for API interactions.
//!
//! ## Architecture
//!
//! - [`api`]: High-level workflow functions (e.g., create PR from workspace)
//! - [`types`]: Workflow-specific types like PrOptions
//!
//! For low-level GitHub API operations, see the `repos-github` crate.

pub mod api;
pub mod types;

// Re-export commonly used items for convenience
pub use api::create_pr_from_workspace;
pub use types::PrOptions;

// Re-export constants for easy access
pub use crate::constants::github::{DEFAULT_BRANCH_PREFIX, DEFAULT_USER_AGENT};
