//! Git operations for pull request workflows
//!
//! This module contains git operations that are commonly used in pull request
//! workflows, including checking for changes, creating branches, staging and
//! committing changes, and pushing branches to remotes.
//!
//! ## Typical PR Workflow
//!
//! 1. [`has_changes`] - Check if repository has uncommitted changes
//! 2. [`create_and_checkout_branch`] - Create and switch to a new branch
//! 3. [`add_all_changes`] - Stage all changes for commit
//! 4. [`commit_changes`] - Commit the staged changes with a message
//! 5. [`push_branch`] - Push the branch to the remote repository
//!
//! ## Additional Utilities
//!
//! - [`get_default_branch`] - Determine the repository's default branch

use anyhow::{Context, Result};
use std::process::Command;

/// Check if a repository has uncommitted changes
pub fn has_changes(repo_path: &str) -> Result<bool> {
    // Check if there are any uncommitted changes using git status
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git status command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to check repository status: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // If output is empty, there are no changes
    Ok(!output.stdout.is_empty())
}

/// Create and checkout a new branch
pub fn create_and_checkout_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    // Create and checkout a new branch using git checkout -b
    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(branch_name)
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git checkout command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create and checkout branch '{}': {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Add all changes to the staging area
pub fn add_all_changes(repo_path: &str) -> Result<()> {
    // Add all changes using git add .
    let output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git add command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to add changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Commit staged changes with a message
pub fn commit_changes(repo_path: &str, message: &str) -> Result<()> {
    // Commit changes using git commit
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git commit command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to commit changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Push a branch to remote and set upstream
pub fn push_branch(repo_path: &str, branch_name: &str) -> Result<()> {
    // Push branch using git push
    let output = Command::new("git")
        .arg("push")
        .arg("--set-upstream")
        .arg("origin")
        .arg(branch_name)
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git push command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to push branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Get the default branch of a repository
pub fn get_default_branch(repo_path: &str) -> Result<String> {
    // Try to get the default branch using git symbolic-ref
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(repo_path)
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let branch_ref = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Some(branch) = branch_ref.strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    // Fallback: try to get the current branch
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_path)
        .output()
        .context("Failed to execute git branch command")?;

    if output.status.success() {
        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !current_branch.is_empty() {
            return Ok(current_branch);
        }
    }

    // Final fallback to default branch
    Ok(crate::constants::git::FALLBACK_BRANCH.to_string())
}
