//! Git operations using system git commands for maximum compatibility

use crate::config::Repository;
use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use std::process::Command;

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

pub fn remove_repository(repo: &Repository) -> Result<()> {
    let target_dir = repo.get_target_dir();

    if Path::new(&target_dir).exists() {
        std::fs::remove_dir_all(&target_dir).context("Failed to remove repository directory")?;
        Ok(())
    } else {
        anyhow::bail!("Repository directory does not exist: {}", target_dir);
    }
}

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
