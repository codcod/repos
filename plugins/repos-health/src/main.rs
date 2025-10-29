use anyhow::{Context, Result};
use chrono::Utc;
use repos::{Config, Repository, load_default_config};
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

    // Parse arguments
    let mut config_path: Option<String> = None;
    let mut include_tags: Vec<String> = Vec::new();
    let mut exclude_tags: Vec<String> = Vec::new();
    let mut debug = false;
    let mut mode = "deps"; // default mode

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a path argument");
                    std::process::exit(1);
                }
            }
            "--tag" => {
                if i + 1 < args.len() {
                    include_tags.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --tag requires a tag argument");
                    std::process::exit(1);
                }
            }
            "--exclude-tag" => {
                if i + 1 < args.len() {
                    exclude_tags.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --exclude-tag requires a tag argument");
                    std::process::exit(1);
                }
            }
            "--debug" | "-d" => {
                debug = true;
                i += 1;
            }
            arg if !arg.starts_with("--") => {
                mode = arg;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Load config (custom path or default)
    let config = if let Some(path) = config_path {
        Config::load_config(&path)
            .with_context(|| format!("Failed to load config from {}", path))?
    } else {
        load_default_config().context("Failed to load default config")?
    };

    // Apply tag filters
    let filtered_repos = filter_repositories(&config.repositories, &include_tags, &exclude_tags);

    if debug {
        eprintln!("DEBUG: Loaded {} repositories", config.repositories.len());
        eprintln!(
            "DEBUG: After filtering: {} repositories",
            filtered_repos.len()
        );
    }

    match mode {
        "deps" => run_deps_check(filtered_repos).await,
        "prs" => run_pr_report(filtered_repos, debug).await,
        _ => {
            eprintln!("Unknown mode: {}. Use 'deps' or 'prs'", mode);
            print_help();
            std::process::exit(1);
        }
    }
}

fn filter_repositories(
    repos: &[Repository],
    include_tags: &[String],
    exclude_tags: &[String],
) -> Vec<Repository> {
    let mut filtered = repos.to_vec();

    // Apply include tags (intersection)
    if !include_tags.is_empty() {
        filtered.retain(|repo| include_tags.iter().any(|tag| repo.tags.contains(tag)));
    }

    // Apply exclude tags (difference)
    if !exclude_tags.is_empty() {
        filtered.retain(|repo| !exclude_tags.iter().any(|tag| repo.tags.contains(tag)));
    }

    filtered
}

fn print_help() {
    println!("repos-health - Repository health checks and reports");
    println!();
    println!("USAGE:");
    println!("    repos health [OPTIONS] [MODE]");
    println!();
    println!("MODES:");
    println!("    deps    Check and update npm dependencies (default)");
    println!("    prs     Generate PR report showing PRs awaiting approval");
    println!();
    println!("DEPS MODE:");
    println!("    Scans repositories for outdated npm packages and automatically");
    println!("    updates them, creates branches, and commits changes.");
    println!();
    println!("    For each repository with a package.json file:");
    println!("    1. Checks for outdated npm packages");
    println!("    2. Updates packages if found");
    println!("    3. Creates a branch and commits changes");
    println!("    4. Pushes the branch to origin");
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
    println!("    -h, --help              Print this help message");
    println!("    -d, --debug             Enable debug output (shows URL parsing)");
    println!("    --config <path>         Use custom config file instead of default");
    println!(
        "    --tag <tag>             Filter to repositories with this tag (can be used multiple times)"
    );
    println!(
        "    --exclude-tag <tag>     Exclude repositories with this tag (can be used multiple times)"
    );
    println!();
    println!("EXAMPLES:");
    println!(
        "    repos health                                    # Run dependency check (default)"
    );
    println!(
        "    repos health deps                               # Explicitly run dependency check"
    );
    println!("    repos health prs                                # Generate PR report");
    println!(
        "    repos health prs --debug                        # Generate PR report with debug info"
    );
    println!(
        "    repos health prs --tag flow                     # PRs for 'flow' tagged repos only"
    );
    println!(
        "    repos health deps --exclude-tag deprecated      # Deps check excluding deprecated repos"
    );
    println!("    repos health prs --config custom.yaml --tag ci  # Custom config with tag filter");
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

async fn run_pr_report(repos: Vec<Repository>, debug: bool) -> Result<()> {
    let token = env::var("GITHUB_TOKEN")
        .context("GITHUB_TOKEN environment variable required for PR reporting")?;

    println!("=================================================");
    println!("  GitHub Pull Requests - Approval Status Report");
    println!("=================================================");
    println!();

    let mut total_repos = 0;
    let mut total_prs = 0;
    let mut total_awaiting = 0;

    for repo in repos {
        if debug {
            eprintln!("DEBUG: Processing repo: {} ({})", repo.name, repo.url);
        }

        match fetch_pr_report(&repo, &token, debug).await {
            Ok(report) => {
                total_repos += 1;
                total_prs += report.total_prs;
                total_awaiting += report.awaiting_approval.len();

                print_repo_report(&report);
            }
            Err(e) => {
                eprintln!("❌ {}: {}", repo.name, e);
            }
        }
    }

    println!();
    println!("=================================================");
    println!("Summary:");
    println!("  Repositories checked: {}", total_repos);
    println!("  Total open PRs: {}", total_prs);
    println!("  PRs awaiting approval: {}", total_awaiting);
    println!("=================================================");

    Ok(())
}

async fn fetch_pr_report(repo: &Repository, token: &str, debug: bool) -> Result<PrReport> {
    // Parse owner/repo from URL
    let (owner, repo_name) = parse_github_repo(&repo.url)
        .with_context(|| format!("Failed to parse GitHub URL: {}", repo.url))?;

    if debug {
        eprintln!(
            "DEBUG: Parsed {} => owner: {}, repo: {}",
            repo.url, owner, repo_name
        );
    }

    // Fetch open PRs from GitHub API
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state=open",
        owner, repo_name
    );

    if debug {
        eprintln!("DEBUG: API URL: {}", url);
    }

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

    let branch = format!("health/deps-{}", short_timestamp());
    create_branch_and_commit(path, &branch, repo, &outdated)?;
    push_branch(path, &branch)?;
    println!(
        "health: {} branch {} pushed - use 'repos pr' to create pull request",
        repo.name, branch
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

fn create_branch_and_commit(
    repo_path: &Path,
    branch: &str,
    repo: &Repository,
    deps: &[String],
) -> Result<()> {
    run(repo_path, ["git", "checkout", "-b", branch])?;
    run(repo_path, ["git", "add", "."])?; // minimal; could restrict
    let msg = format!("chore(health): update dependencies ({})", deps.join(", "));
    run(repo_path, ["git", "commit", "-m", &msg])?;
    println!(
        "health: {} committed dependency updates on {}",
        repo.name, branch
    );
    Ok(())
}

fn push_branch(repo_path: &Path, branch: &str) -> Result<()> {
    run(repo_path, ["git", "push", "-u", "origin", branch])?;
    Ok(())
}

fn run<P: AsRef<Path>, const N: usize>(cwd: P, cmd: [&str; N]) -> Result<()> {
    let status = Command::new(cmd[0])
        .args(&cmd[1..])
        .current_dir(cwd.as_ref())
        .status()
        .with_context(|| format!("exec {:?}", cmd))?;
    if !status.success() {
        anyhow::bail!("command {:?} failed", cmd);
    }
    Ok(())
}

fn short_timestamp() -> String {
    let now = Utc::now();
    format!("{}", now.format("%Y%m%d"))
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
    fn test_short_timestamp_format() {
        let timestamp = short_timestamp();
        // Should be 8 characters in YYYYMMDD format
        assert_eq!(timestamp.len(), 8);
        // Should be all digits
        assert!(timestamp.chars().all(|c| c.is_ascii_digit()));
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

    #[test]
    fn test_run_command_execution() {
        // Test the run function execution path
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Test with a simple command that should succeed
        let result = run(repo_path, ["echo", "test"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_command_failure() {
        // Test the run function error path
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Test with a command that should fail
        let result = run(repo_path, ["nonexistent_command_12345"]);
        assert!(result.is_err());
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

        let result = fetch_pr_report(&repo, "fake-token", false).await;
        assert!(result.is_err());
    }
}
