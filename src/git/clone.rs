//! Git clone and repository removal operations
//!
//! This module handles the core repository lifecycle operations:
//! cloning repositories from remote URLs and removing local repository
//! directories when they're no longer needed.
//!
//! ## Functions
//!
//! - [`clone_repository`]: Clone a repository from its remote URL
//! - [`remove_repository`]: Remove a cloned repository directory
//!
//! Both functions work with the [`Repository`] configuration type and
//! provide detailed logging throughout the operation.

use crate::config::Repository;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::common::Logger;

/// Clone a repository from its URL to the target directory
pub fn clone_repository(repo: &Repository) -> Result<()> {
    let logger = Logger;
    let target_dir = repo.get_target_dir();

    // Check if directory already exists
    if Path::new(&target_dir).exists() {
        logger.warn(repo, "Repository directory already exists, skipping");
        return Ok(());
    }

    let mut args = vec!["clone"];

    // Add branch flag if a branch is specified
    if let Some(branch) = &repo.branch {
        args.extend_from_slice(&["-b", branch]);
        logger.info(
            repo,
            &format!("Cloning branch '{}' from {}", branch, repo.url),
        );
    } else {
        logger.info(repo, &format!("Cloning default branch from {}", repo.url));
    }

    // Add repository URL and target directory
    args.push(&repo.url);
    args.push(&target_dir);

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to execute git clone command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to clone repository: {}", stderr);
    }

    logger.success(repo, "Successfully cloned");
    Ok(())
}

/// Remove a cloned repository directory
pub fn remove_repository(repo: &Repository) -> Result<()> {
    let target_dir = repo.get_target_dir();

    if Path::new(&target_dir).exists() {
        std::fs::remove_dir_all(&target_dir).context("Failed to remove repository directory")?;
        Ok(())
    } else {
        anyhow::bail!("Repository directory does not exist: {}", target_dir);
    }
}
