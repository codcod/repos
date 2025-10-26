//! Git operations using system git commands for maximum compatibility
//!
//! This module is organized into sub-modules for different git workflows:
//!
//! ## Sub-modules
//!
//! - [`clone`]: Repository cloning and removal operations
//!   - `clone_repository()` - Clone a repository from URL
//!   - `remove_repository()` - Remove a cloned repository directory
//!
//! - [`pull_request`]: Git operations specific to pull request workflows
//!   - `has_changes()` - Check for uncommitted changes
//!   - `create_and_checkout_branch()` - Create and switch to new branch
//!   - `add_all_changes()` - Stage all changes
//!   - `commit_changes()` - Commit staged changes
//!   - `push_branch()` - Push branch to remote
//!   - `get_default_branch()` - Get repository's default branch
//!
//! - [`common`]: Shared utilities and helpers
//!   - `Logger` - Consistent logging for git operations
//!
//! ## Benefits of this organization
//!
//! - **Scalability**: Easy to add new git features without making single files unwieldy
//! - **Readability**: Developers can quickly find code for specific git workflows
//! - **Maintainability**: Clear separation of concerns between different git operations
//! - **Backward compatibility**: All functions are re-exported at the module level

pub mod clone;
pub mod common;
pub mod pull_request;

// Re-export all public functions to maintain backward compatibility
pub use clone::{clone_repository, remove_repository};
pub use common::Logger;
pub use pull_request::{
    add_all_changes, commit_changes, create_and_checkout_branch, get_default_branch, has_changes,
    push_branch,
};
