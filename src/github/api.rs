//! GitHub API operations

use super::client::GitHubClient;
use super::types::{PrOptions, PullRequestParams};
use crate::config::Repository;
use crate::git;
use anyhow::Result;
use colored::*;
use uuid::Uuid;

// Constants for maintainability
const DEFAULT_BRANCH_PREFIX: &str = "automated-changes";
const UUID_LENGTH: usize = 6;

/// Create a pull request for a repository
pub async fn create_pull_request(repo: &Repository, options: &PrOptions) -> Result<()> {
    let repo_path = repo.get_target_dir();

    // Check if repository has changes
    if !git::has_changes(&repo_path)? {
        println!(
            "{} | {}",
            repo.name.cyan().bold(),
            "No changes detected".yellow()
        );
        return Ok(());
    }

    // Generate branch name if not provided
    let branch_name = options.branch_name.clone().unwrap_or_else(|| {
        format!(
            "{}-{}",
            DEFAULT_BRANCH_PREFIX,
            &Uuid::new_v4().simple().to_string()[..UUID_LENGTH]
        )
    });

    // Create and checkout new branch
    git::create_and_checkout_branch(&repo_path, &branch_name)?;

    // Add all changes
    git::add_all_changes(&repo_path)?;

    // Commit changes
    let commit_message = options
        .commit_msg
        .clone()
        .unwrap_or_else(|| options.title.clone());
    git::commit_changes(&repo_path, &commit_message)?;

    if !options.create_only {
        // Push branch
        git::push_branch(&repo_path, &branch_name)?;

        // Create PR via GitHub API
        create_github_pr(repo, &branch_name, options).await?;
    }

    Ok(())
}

async fn create_github_pr(repo: &Repository, branch_name: &str, options: &PrOptions) -> Result<()> {
    let client = GitHubClient::new(Some(options.token.clone()));

    // Extract owner and repo name from URL
    let (owner, repo_name) = client.parse_github_url(&repo.url)?;

    // Determine base branch - get actual default branch if not specified
    let base_branch = if let Some(ref base) = options.base_branch {
        base.clone()
    } else {
        git::get_default_branch(&repo.get_target_dir())?
    };

    let result = client
        .create_pull_request(PullRequestParams::new(
            &owner,
            &repo_name,
            &options.title,
            &options.body,
            branch_name,
            &base_branch,
            options.draft,
        ))
        .await?;

    let pr_url = result["html_url"].as_str().unwrap_or("unknown");
    println!(
        "{} | {} {}",
        repo.name.cyan().bold(),
        "Pull request created:".green(),
        pr_url
    );

    Ok(())
}
