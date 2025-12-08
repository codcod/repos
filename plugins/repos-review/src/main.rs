use anyhow::{Context, Result};
use repos::Repository;
use std::env;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let _args: Vec<String> = env::args().collect();

    // Load context injected by core repos CLI
    let repos = repos::load_plugin_context()
        .context("Failed to load plugin context")?
        .ok_or_else(|| anyhow::anyhow!("Plugin must be invoked via repos CLI"))?;

    // Check if fzf is available
    if !is_fzf_available() {
        eprintln!("Error: fzf must be installed.");
        eprintln!("Install it via: brew install fzf (macOS) or your package manager");
        std::process::exit(1);
    }

    // Main loop: select and review repositories
    loop {
        match select_repository(&repos)? {
            Some(repo) => {
                review_repository(&repo)?;
            }
            None => {
                println!("No repo selected. Exiting.");
                break;
            }
        }
    }

    Ok(())
}

/// Check if fzf is installed and available in PATH
fn is_fzf_available() -> bool {
    Command::new("which")
        .arg("fzf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Use fzf to select a repository interactively
fn select_repository(repos: &[Repository]) -> Result<Option<Repository>> {
    // Build list of repository paths for fzf
    let repo_list: Vec<String> = repos
        .iter()
        .filter_map(|r| r.path.as_ref())
        .map(|p| p.to_string())
        .collect();

    if repo_list.is_empty() {
        return Ok(None);
    }

    let input = repo_list.join("\n");

    // Launch fzf with preview showing git status
    let mut fzf = Command::new("fzf")
        .args([
            "--color=fg:#4d4d4c,bg:#eeeeee,hl:#d7005f",
            "--color=fg+:#4d4d4c,bg+:#e8e8e8,hl+:#d7005f",
            "--color=info:#4271ae,prompt:#8959a8,pointer:#d7005f",
            "--color=marker:#4271ae,spinner:#4271ae,header:#4271ae",
            "--height=100%",
            "--ansi",
            "--preview",
            r#"if git -C {} diff-index --quiet HEAD -- 2>/dev/null; then git -C {} status | head -20 | awk 'NF {print "\033[32m" $0 "\033[0m"}'; else git -C {} status | head -20 | awk 'NF {print "\033[31m" $0 "\033[0m"}'; fi"#,
            "--preview-window=right:50%",
            "--no-sort",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn fzf")?;

    // Write repo list to fzf's stdin
    if let Some(mut stdin) = fzf.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .context("Failed to write to fzf stdin")?;
    }

    // Get selected repository path from fzf
    let output = fzf.wait_with_output().context("Failed to wait for fzf")?;

    if !output.status.success() {
        return Ok(None);
    }

    let selected_path = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 from fzf")?
        .trim()
        .to_string();

    if selected_path.is_empty() {
        return Ok(None);
    }

    // Find the matching repository
    let repo = repos
        .iter()
        .find(|r| r.path.as_deref() == Some(selected_path.as_str()))
        .cloned();

    Ok(repo)
}

/// Review a repository by showing git status and git diff
fn review_repository(repo: &Repository) -> Result<()> {
    let repo_path = repo
        .path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Repository has no path"))?;

    // Clear screen
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()?;

    let path_buf = PathBuf::from(repo_path);
    let repo_name = path_buf.file_name().unwrap_or_default().to_string_lossy();

    println!("Reviewing changes in {}...\n", repo_name);

    // Show git status
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .status()
        .context("Failed to run git status")?;

    if !status.success() {
        eprintln!("Warning: git status failed");
    }

    println!();

    // Show git diff
    let diff = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("diff")
        .status()
        .context("Failed to run git diff")?;

    if !diff.success() {
        eprintln!("Warning: git diff failed");
    }

    // Prompt user
    println!("\n\x1b[32mPress [Enter] to go back or [Escape/Q] to exit...\x1b[0m");

    // Read single key
    let mut buffer = [0u8; 1];
    io::stdin()
        .read_exact(&mut buffer)
        .context("Failed to read input")?;

    let key = buffer[0];

    // Check for Escape (27) or Q/q (81/113)
    if key == 27 || key == b'q' || key == b'Q' {
        std::process::exit(0);
    }

    Ok(())
}
