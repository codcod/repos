//! Common git utilities and shared helpers
//!
//! This module contains utilities that are shared across different git workflows,
//! such as logging and error handling helpers.

use crate::config::Repository;
use colored::*;

/// Logger for git operations with consistent formatting
///
/// Provides standardized logging methods for git operations, ensuring
/// consistent output formatting across all git workflows. Each log
/// message is prefixed with the repository name in cyan/bold for
/// easy identification.
///
/// ## Example
///
/// ```rust,no_run
/// use repos::git::Logger;
/// use repos::config::Repository;
///
/// let logger = Logger::default();
/// let repo = Repository::new("my-repo".to_string(), "https://github.com/user/repo.git".to_string());
/// logger.info(&repo, "Starting operation");
/// logger.success(&repo, "Operation completed");
/// ```
#[derive(Default)]
pub struct Logger;

impl Logger {
    pub fn info(&self, repo: &Repository, msg: &str) {
        println!("{} | {}", repo.name.cyan().bold(), msg);
    }

    pub fn success(&self, repo: &Repository, msg: &str) {
        println!("{} | {}", repo.name.cyan().bold(), msg.green());
    }

    pub fn warn(&self, repo: &Repository, msg: &str) {
        println!("{} | {}", repo.name.cyan().bold(), msg.yellow());
    }

    #[allow(dead_code)]
    pub fn error(&self, repo: &Repository, msg: &str) {
        eprintln!("{} | {}", repo.name.cyan().bold(), msg.red());
    }
}
