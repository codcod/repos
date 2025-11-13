use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use repos::{Repository, is_debug_mode, load_plugin_context, save_config};
use repos_github::GitHubClient;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "repos-validate")]
#[command(about = "Validate config.yaml syntax and repository connectivity")]
struct Args {
    /// Validate connectivity to repositories
    #[arg(long)]
    connect: bool,

    /// Synchronize tags with GitHub topics for each repository (requires --connect)
    #[arg(long, requires = "connect")]
    sync_topics: bool,

    /// Apply the topic synchronization to config.yaml (requires --sync-topics)
    #[arg(long, requires = "sync_topics")]
    apply: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let debug = is_debug_mode();

    // Load repositories from injected context or fail
    let repos = load_plugin_context()?
        .context("Failed to load plugin context. Make sure to run this via 'repos validate'")?;

    if debug {
        eprintln!("Loaded {} repositories from context", repos.len());
    }

    println!("{}", "✅ config.yaml syntax is valid.".green());
    println!();

    if !args.connect {
        println!("{}", "Validation finished successfully.".green());
        return Ok(());
    }

    println!("Validating repository connectivity...");

    let gh_client = GitHubClient::new(None);
    let mut errors = 0;
    let mut sync_map: HashMap<String, TopicSync> = HashMap::new();

    for repo in repos {
        match validate_repository(&gh_client, &repo, args.sync_topics).await {
            Ok(topics) => {
                println!("{} {}: Accessible.", "✅".green(), repo.name);
                if args.sync_topics && !topics.is_empty() {
                    let existing_tags: HashSet<_> = repo.tags.iter().cloned().collect();

                    // GitHub topics with gh: prefix
                    let gh_topics: HashSet<String> =
                        topics.iter().map(|t| format!("gh:{}", t)).collect();

                    // Find existing gh: tags in config
                    let existing_gh_tags: HashSet<String> = existing_tags
                        .iter()
                        .filter(|t| t.starts_with("gh:"))
                        .cloned()
                        .collect();

                    // Topics to add (in GitHub but not in tags)
                    let to_add: Vec<String> =
                        gh_topics.difference(&existing_tags).cloned().collect();

                    // Topics to remove (gh: tags in config but not in GitHub topics)
                    let to_remove: Vec<String> =
                        existing_gh_tags.difference(&gh_topics).cloned().collect();

                    if !to_add.is_empty() || !to_remove.is_empty() {
                        if args.apply {
                            sync_map.insert(
                                repo.name.clone(),
                                TopicSync {
                                    add: to_add.clone(),
                                    remove: to_remove.clone(),
                                },
                            );
                            if !to_add.is_empty() {
                                println!("    - Topics to add: {:?}", to_add);
                            }
                            if !to_remove.is_empty() {
                                println!("    - Topics to remove: {:?}", to_remove);
                            }
                        } else {
                            if !to_add.is_empty() {
                                println!("    - Would add: {:?}", to_add);
                            }
                            if !to_remove.is_empty() {
                                println!("    - Would remove: {:?}", to_remove);
                            }
                        }
                    } else {
                        println!("    - Topics already synchronized");
                    }
                }
            }
            Err(e) => {
                println!("{} {}: {}", "❌".red(), repo.name, e);
                errors += 1;
            }
        }
    }

    println!();
    if errors > 0 {
        println!(
            "{}",
            format!("Validation finished with {} error(s).", errors).red()
        );
        std::process::exit(1);
    } else {
        println!("{}", "Validation finished successfully.".green());
    }

    // Apply supplementation if requested
    if args.apply && !sync_map.is_empty() {
        println!();
        let config_path = get_config_path()?;
        apply_sync(&config_path, &sync_map)?;
    }

    Ok(())
}

#[derive(Debug)]
struct TopicSync {
    add: Vec<String>,
    remove: Vec<String>,
}

async fn validate_repository(
    gh_client: &GitHubClient,
    repo: &Repository,
    fetch_topics: bool,
) -> Result<Vec<String>> {
    // Parse owner/repo from the URL
    let (owner, repo_name) = parse_github_url(&repo.url)?;

    // Get repository details from GitHub
    let repo_data = gh_client.get_repository_details(&owner, &repo_name).await?;

    // Return topics if requested, otherwise empty vector
    if fetch_topics {
        Ok(repo_data.topics)
    } else {
        Ok(vec![])
    }
}

fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle SSH URLs: git@github.com:owner/repo.git
    if url.starts_with("git@github.com:") {
        let path = url
            .strip_prefix("git@github.com:")
            .context("Invalid GitHub SSH URL")?
            .strip_suffix(".git")
            .unwrap_or(url.strip_prefix("git@github.com:").unwrap());

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid GitHub repository path: {}", path);
        }
        return Ok((parts[0].to_string(), parts[1].to_string()));
    }

    // Handle HTTPS URLs: https://github.com/owner/repo.git
    if url.starts_with("https://github.com/") {
        let path = url
            .strip_prefix("https://github.com/")
            .context("Invalid GitHub HTTPS URL")?
            .strip_suffix(".git")
            .unwrap_or(url.strip_prefix("https://github.com/").unwrap());

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid GitHub repository path: {}", path);
        }
        return Ok((parts[0].to_string(), parts[1].to_string()));
    }

    anyhow::bail!("Unsupported repository URL format: {}", url)
}

fn get_config_path() -> Result<PathBuf> {
    // Try to get config path from environment variable set by repos CLI
    if let Ok(config) = std::env::var("REPOS_CONFIG_FILE") {
        let debug = is_debug_mode();
        if debug {
            eprintln!("Using config file from REPOS_CONFIG_FILE: {}", config);
        }
        return Ok(PathBuf::from(config));
    }

    // Default to config.yaml in current directory
    Ok(PathBuf::from("config.yaml"))
}

fn create_backup(config_path: &PathBuf) -> Result<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = config_path.with_extension(format!("yaml.backup.{}", timestamp));

    fs::copy(config_path, &backup_path)
        .context(format!("Failed to create backup at {:?}", backup_path))?;

    println!("{} Created backup: {:?}", "✅".green(), backup_path);
    Ok(backup_path)
}

fn apply_sync(config_path: &PathBuf, sync_map: &HashMap<String, TopicSync>) -> Result<()> {
    println!("Applying topic synchronization to config.yaml...");

    // Create backup first
    create_backup(config_path)?;

    // Read the config file
    let content = fs::read_to_string(config_path)
        .context(format!("Failed to read config file: {:?}", config_path))?;

    // Parse as YAML
    let mut config: serde_yaml::Value =
        serde_yaml::from_str(&content).context("Failed to parse config.yaml")?;

    // Update repositories with synchronized topics
    if let Some(repos) = config
        .get_mut("repositories")
        .and_then(|r| r.as_sequence_mut())
    {
        for repo in repos {
            // Get the name first
            let name = repo.get("name").and_then(|n| n.as_str()).map(String::from);

            if let Some(name) = name
                && let Some(sync) = sync_map.get(&name)
            {
                // Get existing tags or create new array
                let tags = repo
                    .get_mut("tags")
                    .and_then(|t| t.as_sequence_mut())
                    .context(format!("Repository '{}' has invalid tags field", name))?;

                // Remove outdated gh: tags
                for topic in &sync.remove {
                    let topic_value = serde_yaml::Value::String(topic.clone());
                    if let Some(pos) = tags.iter().position(|t| t == &topic_value) {
                        tags.remove(pos);
                    }
                }

                // Add new topics
                for topic in &sync.add {
                    let topic_value = serde_yaml::Value::String(topic.clone());
                    if !tags.contains(&topic_value) {
                        tags.push(topic_value);
                    }
                }
            }
        }
    }

    // Write back to file using centralized save_config function
    save_config(&config, config_path.to_str().unwrap())
        .context("Failed to write updated config")?;

    println!("{} Successfully updated config.yaml", "✅".green());
    println!("   {} repositories were synchronized", sync_map.len());

    Ok(())
}
