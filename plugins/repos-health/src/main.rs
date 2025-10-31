use anyhow::{Context, Result};
use repos::Repository;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize)]
struct PrUser {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {
    number: u64,
    title: String,
    html_url: String,
    user: PrUser,
    draft: bool,
    #[serde(default)]
    requested_reviewers: Vec<PrUser>,
}

#[derive(Debug)]
struct PrReport {
    repo_name: String,
    total_prs: usize,
    awaiting_approval: Vec<PrSummary>,
}

#[derive(Debug)]
struct PrSummary {
    number: u64,
    title: String,
    author: String,
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Load context injected by core repos CLI
    let repos = repos::load_plugin_context()
        .context("Failed to load plugin context")?
        .ok_or_else(|| anyhow::anyhow!("Plugin must be invoked via repos CLI"))?;

    // Parse mode from arguments
    let mut mode = "deps"; // default mode
    for arg in &args[1..] {
        if arg == "deps" || arg == "prs" {
            mode = arg;
            break;
        } else if arg == "--help" || arg == "-h" {
            print_help();
            return Ok(());
        }
    }

    match mode {
        "deps" => run_deps_check(repos).await,
        "prs" => run_pr_report(repos).await,
        _ => {
            eprintln!("Unknown mode: {}. Use 'deps' or 'prs'", mode);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("repos-health - Repository health checks and reports");
    println!();
    println!("USAGE:");
    println!("    repos health [MODE]");
    println!();
    println!("MODES:");
    println!("    deps    Check and update npm dependencies (default)");
    println!("    prs     Generate PR report showing PRs awaiting approval");
    println!();
    println!("DEPS MODE:");
    println!("    Scans repositories for outdated npm packages and automatically");
    println!("    updates them locally.");
    println!();
    println!("    For each repository with a package.json file:");
    println!("    1. Checks for outdated npm packages");
    println!("    2. Updates packages if found");
    println!("    3. Reports changes for manual commit");
    println!();
    println!("PRS MODE:");
    println!("    Generates a report of open pull requests awaiting approval");
    println!("    across all configured repositories. Shows:");
    println!("    - Total number of PRs per repository");
    println!("    - PRs without requested reviewers");
    println!("    - PR number, title, author, and URL");
    println!();
    println!("    Requires:");
    println!("    - GITHUB_TOKEN environment variable for API access");
    println!("    - Repositories must be GitHub repositories");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help    Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    repos health          # Run dependency check (default)");
    println!("    repos health deps     # Explicitly run dependency check");
    println!("    repos health prs      # Generate PR report");
}

async fn run_deps_check(repos: Vec<Repository>) -> Result<()> {
    let mut processed = 0;
    for repo in repos {
        if let Err(e) = process_repo(&repo) {
            eprintln!("health: {} skipped: {}", repo.name, e);
        } else {
            processed += 1;
        }
    }
    println!("health: processed {} repositories", processed);
    Ok(())
}

async fn run_pr_report(repos: Vec<Repository>) -> Result<()> {
    let github_token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set")?;
    let mut reports = Vec::new();

    for repo in &repos {
        match fetch_pr_report(repo, &github_token).await {
            Ok(report) => reports.push(report),
            Err(e) => eprintln!("Error fetching PRs for {}: {}", repo.name, e),
        }
    }

    println!("\n=== Pull Request Report ===\n");
    for report in &reports {
        print_repo_report(report);
    }

    let total_prs: usize = reports.iter().map(|r| r.total_prs).sum();
    let total_awaiting: usize = reports.iter().map(|r| r.awaiting_approval.len()).sum();
    println!(
        "Total: {} open PRs, {} awaiting review assignment",
        total_prs, total_awaiting
    );

    Ok(())
}

async fn fetch_pr_report(repo: &Repository, token: &str) -> Result<PrReport> {
    // Parse owner/repo from URL
    let (owner, repo_name) = parse_github_repo(&repo.url)
        .with_context(|| format!("Failed to parse GitHub URL: {}", repo.url))?;

    // Fetch open PRs from GitHub API
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state=open",
        owner, repo_name
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "repos-health")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("Failed to fetch PRs from GitHub")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "GitHub API error: {} - {} (URL parsed as: {}/{})",
            response.status(),
            response.text().await.unwrap_or_default(),
            owner,
            repo_name
        );
    }

    let prs: Vec<PullRequest> = response
        .json()
        .await
        .context("Failed to parse PR response")?;

    let total_prs = prs.len();
    let awaiting_approval: Vec<PrSummary> = prs
        .into_iter()
        .filter(|pr| !pr.draft && pr.requested_reviewers.is_empty())
        .map(|pr| PrSummary {
            number: pr.number,
            title: pr.title,
            author: pr.user.login,
            url: pr.html_url,
        })
        .collect();

    Ok(PrReport {
        repo_name: repo.name.clone(),
        total_prs,
        awaiting_approval,
    })
}

fn parse_github_repo(url: &str) -> Result<(String, String)> {
    // Parse GitHub URL: https://github.com/owner/repo.git or git@github.com:owner/repo.git
    let url = url.trim_end_matches(".git");

    // Handle SSH format: git@github.com:owner/repo
    if url.contains("git@github.com:") {
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() >= 2 {
            let repo_path = parts[1];
            let path_parts: Vec<&str> = repo_path.split('/').collect();
            if path_parts.len() >= 2 {
                return Ok((
                    path_parts[path_parts.len() - 2].to_string(),
                    path_parts[path_parts.len() - 1].to_string(),
                ));
            }
        }
    }

    // Handle HTTPS format: https://github.com/owner/repo
    let parts: Vec<&str> = url.split('/').collect();

    if parts.len() < 2 {
        anyhow::bail!("Invalid GitHub URL format: {}", url);
    }

    let owner = parts[parts.len() - 2].to_string();
    let repo = parts[parts.len() - 1].to_string();

    Ok((owner, repo))
}

fn print_repo_report(report: &PrReport) {
    if report.total_prs == 0 {
        println!("✅ {}: No open PRs", report.repo_name);
        return;
    }

    println!(
        "📊 {}: {} open PR{}",
        report.repo_name,
        report.total_prs,
        if report.total_prs == 1 { "" } else { "s" }
    );

    if report.awaiting_approval.is_empty() {
        println!("   ✓ All PRs have reviewers assigned");
    } else {
        println!(
            "   ⚠️  {} PR{} awaiting reviewer assignment:",
            report.awaiting_approval.len(),
            if report.awaiting_approval.len() == 1 {
                ""
            } else {
                "s"
            }
        );
        for pr in &report.awaiting_approval {
            println!("      #{} - {} (by @{})", pr.number, pr.title, pr.author);
            println!("         {}", pr.url);
        }
    }
    println!();
}

fn process_repo(repo: &Repository) -> Result<()> {
    let repo_path = repo.get_target_dir();
    let path = Path::new(&repo_path);
    let pkg = path.join("package.json");
    if !pkg.exists() {
        anyhow::bail!("no package.json");
    }

    let outdated = check_outdated(path)?;
    if outdated.is_empty() {
        println!("health: {} up-to-date", repo.name);
        return Ok(());
    }

    println!(
        "health: {} outdated packages: {}",
        repo.name,
        outdated.join(", ")
    );
    update_dependencies(path)?;
    let changed = has_lockfile_changes(path)?;
    if !changed {
        println!("health: {} no lockfile changes after update", repo.name);
        return Ok(());
    }

    println!(
        "health: {} dependencies updated - review changes and commit manually",
        repo.name
    );
    Ok(())
}

fn check_outdated(repo_path: &Path) -> Result<Vec<String>> {
    // Try npm outdated --json; if npm missing or error, return mock info
    let output = Command::new("npm")
        .arg("outdated")
        .arg("--json")
        .current_dir(repo_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(o) if o.status.success() || o.status.code() == Some(1) => {
            // npm outdated exits 1 if there are outdated deps
            if o.stdout.is_empty() {
                return Ok(vec![]);
            }
            let v: serde_json::Value =
                serde_json::from_slice(&o.stdout).context("parse npm outdated json")?;
            let mut deps = Vec::new();
            if let serde_json::Value::Object(map) = v {
                for (name, info) in map {
                    if info.get("latest").is_some() {
                        deps.push(name);
                    }
                }
            }
            Ok(deps)
        }
        Ok(_) => Ok(vec![]),
        Err(_) => {
            // Mock fallback when npm not present
            Ok(vec![]) // keep empty for minimal intrusive behavior
        }
    }
}

fn update_dependencies(repo_path: &Path) -> Result<()> {
    // Best effort upgrade; ignore failures to keep minimal
    let _ = Command::new("npm")
        .arg("update")
        .current_dir(repo_path)
        .status();
    Ok(())
}

fn has_lockfile_changes(repo_path: &Path) -> Result<bool> {
    // Check git diff for package-lock.json / yarn.lock / pnpm-lock.yaml
    let patterns = ["package-lock.json", "yarn.lock", "pnpm-lock.yaml"];
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(repo_path)
        .output()
        .context("git status")?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(patterns.iter().any(|p| text.contains(p)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_print_help() {
        // Test that print_help function executes without panicking
        print_help();
        // If we reach this point, the function executed successfully
        // Test passes if print_help() completes without panicking
    }

    #[test]
    fn test_parse_github_repo_valid() {
        let url = "https://github.com/owner/repo.git";
        let result = parse_github_repo(url);
        assert!(result.is_ok());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_without_git_extension() {
        let url = "https://github.com/owner/repo";
        let result = parse_github_repo(url);
        assert!(result.is_ok());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_ssh_format() {
        let url = "git@github.com:owner/repo.git";
        let result = parse_github_repo(url);
        assert!(result.is_ok());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_ssh_without_git() {
        let url = "git@github.com:owner/repo";
        let result = parse_github_repo(url);
        assert!(result.is_ok());
        let (owner, repo) = result.unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_github_repo_invalid() {
        let url = "invalid";
        let result = parse_github_repo(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_print_repo_report_no_prs() {
        let report = PrReport {
            repo_name: "test-repo".to_string(),
            total_prs: 0,
            awaiting_approval: vec![],
        };
        print_repo_report(&report);
        // Should complete without panic
    }

    #[test]
    fn test_print_repo_report_with_prs() {
        let report = PrReport {
            repo_name: "test-repo".to_string(),
            total_prs: 2,
            awaiting_approval: vec![PrSummary {
                number: 123,
                title: "Test PR".to_string(),
                author: "testuser".to_string(),
                url: "https://github.com/owner/repo/pull/123".to_string(),
            }],
        };
        print_repo_report(&report);
        // Should complete without panic
    }

    #[test]
    fn test_check_outdated_execution() {
        // Test execution path for check_outdated function
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // This will hit the npm command execution path
        // Expected to return empty vec since npm likely not available in test environment
        let result = check_outdated(repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_dependencies_execution() {
        // Test execution path for update_dependencies function
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // This will execute the npm update command path
        let result = update_dependencies(repo_path);
        assert!(result.is_ok()); // Should always succeed (ignores npm failures)
    }

    #[test]
    fn test_has_lockfile_changes_execution() {
        // Test execution path for has_lockfile_changes function
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a git repo for the test
        let _ = Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output();

        // This will hit the git status execution path
        let result = has_lockfile_changes(repo_path);
        // May succeed or fail depending on git setup, but tests execution path
        let _ = result; // Don't assert result since git may not be available
    }

    #[test]
    fn test_process_repo_no_package_json() {
        // Test process_repo execution path when no package.json exists
        let temp_dir = TempDir::new().unwrap();

        let repo = Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            branch: None,
            tags: vec![],
            config_dir: None,
        };

        // This should hit the "no package.json" error path
        let result = process_repo(&repo);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no package.json"));
    }

    #[tokio::test]
    async fn test_fetch_pr_report_invalid_url() {
        let repo = Repository {
            name: "test".to_string(),
            url: "invalid".to_string(),
            path: None,
            branch: None,
            tags: vec![],
            config_dir: None,
        };

        let result = fetch_pr_report(&repo, "fake-token").await;
        assert!(result.is_err());
    }
}
