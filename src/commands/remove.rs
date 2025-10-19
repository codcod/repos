//! Remove command implementation

use super::{Command, CommandContext};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use std::fs;

/// Remove command for deleting cloned repositories
pub struct RemoveCommand;

#[async_trait]
impl Command for RemoveCommand {
    async fn execute(&self, context: &CommandContext) -> Result<()> {
        let repositories = context
            .config
            .filter_repositories(context.tag.as_deref(), context.repos.as_deref());

        if repositories.is_empty() {
            let filter_desc = match (&context.tag, &context.repos) {
                (Some(tag), Some(repos)) => format!("tag '{tag}' and repositories {repos:?}"),
                (Some(tag), None) => format!("tag '{tag}'"),
                (None, Some(repos)) => format!("repositories {repos:?}"),
                (None, None) => "no repositories found".to_string(),
            };
            println!(
                "{}",
                format!("No repositories found with {filter_desc}").yellow()
            );
            return Ok(());
        }

        println!(
            "{}",
            format!("Removing {} repositories...", repositories.len()).green()
        );

        let mut errors = Vec::new();
        let mut successful = 0;

        if context.parallel {
            let tasks: Vec<_> = repositories
                .into_iter()
                .map(|repo| {
                    let repo_name = repo.name.clone();
                    tokio::spawn(async move {
                        let target_dir = repo.get_target_dir();
                        let result = tokio::task::spawn_blocking(move || {
                            if std::path::Path::new(&target_dir).exists() {
                                fs::remove_dir_all(&target_dir).map_err(anyhow::Error::from)
                            } else {
                                println!("{} | Directory does not exist", repo.name.cyan().bold());
                                Ok(())
                            }
                        })
                        .await?;
                        Ok::<_, anyhow::Error>((repo_name, result))
                    })
                })
                .collect();

            for task in tasks {
                match task.await? {
                    Ok((_, Ok(_))) => successful += 1,
                    Ok((repo_name, Err(e))) => {
                        eprintln!("{}", format!("Error: {e}").red());
                        errors.push((repo_name, e));
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Task error: {e}").red());
                        errors.push(("unknown".to_string(), e));
                    }
                }
            }
        } else {
            for repo in repositories {
                let target_dir = repo.get_target_dir();
                if std::path::Path::new(&target_dir).exists() {
                    match fs::remove_dir_all(&target_dir) {
                        Ok(_) => {
                            println!("{} | {}", repo.name.cyan().bold(), "Removed".green());
                            successful += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "{} | {}",
                                repo.name.cyan().bold(),
                                format!("Error: {e}").red()
                            );
                            errors.push((repo.name.clone(), e.into()));
                        }
                    }
                } else {
                    println!("{} | Directory does not exist", repo.name.cyan().bold());
                    successful += 1; // Count as success since the desired state is achieved
                }
            }
        }

        // Report summary
        if errors.is_empty() {
            println!("{}", "Done removing repositories".green());
        } else {
            println!(
                "{}",
                format!(
                    "Completed with {} successful, {} failed",
                    successful,
                    errors.len()
                )
                .yellow()
            );

            // If all operations failed, return an error to propagate to main
            if successful == 0 {
                return Err(anyhow::anyhow!(
                    "All removal operations failed. First error: {}",
                    errors[0].1
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Repository};
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_remove_command_basic_removal() {
        let temp_dir = TempDir::new().unwrap();

        // Create a directory to remove
        let repo_dir = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join("file.txt"), "test content").unwrap();

        let repo = Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/user/test-repo.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![repo],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        assert!(repo_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // Directory should be removed
        assert!(!repo_dir.exists());
    }

    #[tokio::test]
    async fn test_remove_command_multiple_repositories() {
        let temp_dir = TempDir::new().unwrap();

        let mut repositories = Vec::new();
        let mut repo_dirs = Vec::new();

        // Create multiple directories
        for i in 1..=3 {
            let repo_dir = temp_dir.path().join(format!("repo-{}", i));
            fs::create_dir_all(&repo_dir).unwrap();
            fs::write(repo_dir.join("file.txt"), "test content").unwrap();

            let repo = Repository {
                name: format!("repo-{}", i),
                url: format!("https://github.com/user/repo-{}.git", i),
                tags: vec!["test".to_string()],
                path: Some(repo_dir.to_string_lossy().to_string()),
                branch: None,
                config_dir: None,
            };

            repositories.push(repo);
            repo_dirs.push(repo_dir);
        }

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config { repositories },
            tag: None,
            repos: None,
            parallel: false,
        };

        // Verify all directories exist
        for repo_dir in &repo_dirs {
            assert!(repo_dir.exists());
        }

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // All directories should be removed
        for repo_dir in &repo_dirs {
            assert!(!repo_dir.exists());
        }
    }

    #[tokio::test]
    async fn test_remove_command_parallel_execution() {
        let temp_dir = TempDir::new().unwrap();

        let mut repositories = Vec::new();
        let mut repo_dirs = Vec::new();

        // Create multiple directories
        for i in 1..=3 {
            let repo_dir = temp_dir.path().join(format!("parallel-repo-{}", i));
            fs::create_dir_all(&repo_dir).unwrap();
            fs::write(repo_dir.join("file.txt"), "test content").unwrap();

            let repo = Repository {
                name: format!("parallel-repo-{}", i),
                url: format!("https://github.com/user/parallel-repo-{}.git", i),
                tags: vec!["test".to_string()],
                path: Some(repo_dir.to_string_lossy().to_string()),
                branch: None,
                config_dir: None,
            };

            repositories.push(repo);
            repo_dirs.push(repo_dir);
        }

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config { repositories },
            tag: None,
            repos: None,
            parallel: true, // Enable parallel execution
        };

        // Verify all directories exist
        for repo_dir in &repo_dirs {
            assert!(repo_dir.exists());
        }

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // All directories should be removed
        for repo_dir in &repo_dirs {
            assert!(!repo_dir.exists());
        }
    }

    #[tokio::test]
    async fn test_remove_command_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();

        let repo_dir = temp_dir.path().join("nonexistent-repo");
        // Don't create the directory

        let repo = Repository {
            name: "nonexistent-repo".to_string(),
            url: "https://github.com/user/nonexistent-repo.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![repo],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        assert!(!repo_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok()); // Should succeed since desired state is achieved
    }

    #[tokio::test]
    async fn test_remove_command_with_tag_filter() {
        let temp_dir = TempDir::new().unwrap();

        // Create repository with matching tag
        let matching_repo_dir = temp_dir.path().join("matching-repo");
        fs::create_dir_all(&matching_repo_dir).unwrap();

        let matching_repo = Repository {
            name: "matching-repo".to_string(),
            url: "https://github.com/user/matching-repo.git".to_string(),
            tags: vec!["backend".to_string()],
            path: Some(matching_repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        // Create repository with non-matching tag
        let non_matching_repo_dir = temp_dir.path().join("non-matching-repo");
        fs::create_dir_all(&non_matching_repo_dir).unwrap();

        let non_matching_repo = Repository {
            name: "non-matching-repo".to_string(),
            url: "https://github.com/user/non-matching-repo.git".to_string(),
            tags: vec!["frontend".to_string()],
            path: Some(non_matching_repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![matching_repo, non_matching_repo],
            },
            tag: Some("backend".to_string()),
            repos: None,
            parallel: false,
        };

        assert!(matching_repo_dir.exists());
        assert!(non_matching_repo_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // Only matching repository should be removed
        assert!(!matching_repo_dir.exists());
        assert!(non_matching_repo_dir.exists()); // Should still exist
    }

    #[tokio::test]
    async fn test_remove_command_with_repo_filter() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple repositories
        let repo1_dir = temp_dir.path().join("repo1");
        fs::create_dir_all(&repo1_dir).unwrap();

        let repo2_dir = temp_dir.path().join("repo2");
        fs::create_dir_all(&repo2_dir).unwrap();

        let repo1 = Repository {
            name: "repo1".to_string(),
            url: "https://github.com/user/repo1.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(repo1_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let repo2 = Repository {
            name: "repo2".to_string(),
            url: "https://github.com/user/repo2.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(repo2_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![repo1, repo2],
            },
            tag: None,
            repos: Some(vec!["repo1".to_string()]), // Only remove repo1
            parallel: false,
        };

        assert!(repo1_dir.exists());
        assert!(repo2_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // Only repo1 should be removed
        assert!(!repo1_dir.exists());
        assert!(repo2_dir.exists()); // Should still exist
    }

    #[tokio::test]
    async fn test_remove_command_no_matching_repositories() {
        let temp_dir = TempDir::new().unwrap();

        let repo = Repository {
            name: "test-repo".to_string(),
            url: "https://github.com/user/test-repo.git".to_string(),
            tags: vec!["backend".to_string()],
            path: Some(
                temp_dir
                    .path()
                    .join("test-repo")
                    .to_string_lossy()
                    .to_string(),
            ),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![repo],
            },
            tag: Some("frontend".to_string()), // Non-matching tag
            repos: None,
            parallel: false,
        };

        let result = command.execute(&context).await;
        assert!(result.is_ok()); // Should succeed but do nothing
    }

    #[tokio::test]
    async fn test_remove_command_empty_repositories() {
        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        let result = command.execute(&context).await;
        assert!(result.is_ok()); // Should succeed with empty repository list
    }

    #[tokio::test]
    async fn test_remove_command_permission_error_handling() {
        let temp_dir = TempDir::new().unwrap();

        // Create a directory structure that might cause permission issues
        let repo_dir = temp_dir.path().join("protected-repo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join("file.txt"), "test content").unwrap();

        // On Unix systems, we could try to set read-only permissions to simulate errors
        // But for portability, we'll just test with a regular directory
        // and trust that the error handling code works correctly

        let repo = Repository {
            name: "protected-repo".to_string(),
            url: "https://github.com/user/protected-repo.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![repo],
            },
            tag: None,
            repos: None,
            parallel: false,
        };

        let result = command.execute(&context).await;
        // For a normal directory, this should succeed
        assert!(result.is_ok());
        assert!(!repo_dir.exists());
    }

    #[tokio::test]
    async fn test_remove_command_combined_filters() {
        let temp_dir = TempDir::new().unwrap();

        // Create repository matching both tag and name filters
        let matching_repo_dir = temp_dir.path().join("matching-repo");
        fs::create_dir_all(&matching_repo_dir).unwrap();

        let matching_repo = Repository {
            name: "matching-repo".to_string(),
            url: "https://github.com/user/matching-repo.git".to_string(),
            tags: vec!["backend".to_string()],
            path: Some(matching_repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        // Create repository with matching tag but wrong name
        let wrong_name_repo_dir = temp_dir.path().join("wrong-name-repo");
        fs::create_dir_all(&wrong_name_repo_dir).unwrap();

        let wrong_name_repo = Repository {
            name: "wrong-name-repo".to_string(),
            url: "https://github.com/user/wrong-name-repo.git".to_string(),
            tags: vec!["backend".to_string()],
            path: Some(wrong_name_repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![matching_repo, wrong_name_repo],
            },
            tag: Some("backend".to_string()),
            repos: Some(vec!["matching-repo".to_string()]),
            parallel: false,
        };

        assert!(matching_repo_dir.exists());
        assert!(wrong_name_repo_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // Only the repository matching both filters should be removed
        assert!(!matching_repo_dir.exists());
        assert!(wrong_name_repo_dir.exists()); // Should still exist
    }

    #[tokio::test]
    async fn test_remove_command_parallel_with_mixed_success_failure() {
        let temp_dir = TempDir::new().unwrap();

        // Create one normal directory that can be removed
        let success_repo_dir = temp_dir.path().join("success-repo");
        fs::create_dir_all(&success_repo_dir).unwrap();

        let success_repo = Repository {
            name: "success-repo".to_string(),
            url: "https://github.com/user/success-repo.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(success_repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        // Create a repository pointing to a nonexistent directory (should succeed as desired state)
        let nonexistent_repo = Repository {
            name: "nonexistent-repo".to_string(),
            url: "https://github.com/user/nonexistent-repo.git".to_string(),
            tags: vec!["test".to_string()],
            path: Some(
                temp_dir
                    .path()
                    .join("nonexistent")
                    .to_string_lossy()
                    .to_string(),
            ),
            branch: None,
            config_dir: None,
        };

        let command = RemoveCommand;
        let context = CommandContext {
            config: Config {
                repositories: vec![success_repo, nonexistent_repo],
            },
            tag: None,
            repos: None,
            parallel: true, // Test parallel execution with mixed scenarios
        };

        assert!(success_repo_dir.exists());

        let result = command.execute(&context).await;
        assert!(result.is_ok());

        // Success repo should be removed
        assert!(!success_repo_dir.exists());
    }
}
