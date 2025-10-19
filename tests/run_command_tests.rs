use repos::{
    commands::{Command, CommandContext, run::RunCommand},
    config::{Config, Repository},
};
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

/// Helper function to create a git repository in a directory
fn create_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    // Initialize git repo
    ProcessCommand::new("git")
        .arg("init")
        .current_dir(path)
        .output()?;

    // Configure git (required for commits)
    ProcessCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    ProcessCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    // Create a file and commit
    fs::write(path.join("README.md"), "# Test Repository")?;

    ProcessCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;

    ProcessCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

#[tokio::test]
async fn test_run_command_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create a test repository directory
    let repo_dir = temp_dir.path().join("test-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/user/test-repo.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: None,
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_multiple_repositories() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let mut repositories = Vec::new();

    // Create multiple test repositories
    for i in 1..=3 {
        let repo_dir = temp_dir.path().join(format!("repo-{}", i));
        fs::create_dir_all(&repo_dir).unwrap();
        create_git_repo(&repo_dir).unwrap();

        let repo = Repository {
            name: format!("repo-{}", i),
            url: format!("https://github.com/user/repo-{}.git", i),
            tags: vec!["test".to_string()],
            path: Some(repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        repositories.push(repo);
    }

    let command = RunCommand {
        command: "pwd".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config { repositories },
        tag: None,
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_parallel_execution() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let mut repositories = Vec::new();

    // Create multiple test repositories
    for i in 1..=3 {
        let repo_dir = temp_dir.path().join(format!("parallel-repo-{}", i));
        fs::create_dir_all(&repo_dir).unwrap();
        create_git_repo(&repo_dir).unwrap();

        let repo = Repository {
            name: format!("parallel-repo-{}", i),
            url: format!("https://github.com/user/parallel-repo-{}.git", i),
            tags: vec!["test".to_string()],
            path: Some(repo_dir.to_string_lossy().to_string()),
            branch: None,
            config_dir: None,
        };

        repositories.push(repo);
    }

    let command = RunCommand {
        command: "echo parallel".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config { repositories },
        tag: None,
        repos: None,
        parallel: true, // Enable parallel execution
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_tag_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create repository with matching tag
    let matching_repo_dir = temp_dir.path().join("backend-repo");
    fs::create_dir_all(&matching_repo_dir).unwrap();
    create_git_repo(&matching_repo_dir).unwrap();

    let matching_repo = Repository {
        name: "backend-repo".to_string(),
        url: "https://github.com/user/backend-repo.git".to_string(),
        tags: vec!["backend".to_string()],
        path: Some(matching_repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    // Create repository with non-matching tag
    let non_matching_repo_dir = temp_dir.path().join("frontend-repo");
    fs::create_dir_all(&non_matching_repo_dir).unwrap();
    create_git_repo(&non_matching_repo_dir).unwrap();

    let non_matching_repo = Repository {
        name: "frontend-repo".to_string(),
        url: "https://github.com/user/frontend-repo.git".to_string(),
        tags: vec!["frontend".to_string()],
        path: Some(non_matching_repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo tagged".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![matching_repo, non_matching_repo],
        },
        tag: Some("backend".to_string()),
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_repo_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create multiple repositories
    let repo1_dir = temp_dir.path().join("repo1");
    fs::create_dir_all(&repo1_dir).unwrap();
    create_git_repo(&repo1_dir).unwrap();

    let repo2_dir = temp_dir.path().join("repo2");
    fs::create_dir_all(&repo2_dir).unwrap();
    create_git_repo(&repo2_dir).unwrap();

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

    let command = RunCommand {
        command: "echo filtered".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo1, repo2],
        },
        tag: None,
        repos: Some(vec!["repo1".to_string()]), // Only run on repo1
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_no_matching_repositories() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

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

    let command = RunCommand {
        command: "echo test".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

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
async fn test_run_command_empty_repositories() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let command = RunCommand {
        command: "echo test".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

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
async fn test_run_command_complex_command() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create a test repository directory
    let repo_dir = temp_dir.path().join("complex-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: "complex-repo".to_string(),
        url: "https://github.com/user/complex-repo.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "git status && echo done".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: None,
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_command_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create a test repository directory
    let repo_dir = temp_dir.path().join("special-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: "special-repo".to_string(),
        url: "https://github.com/user/special-repo.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo 'hello world' && echo \"quoted text\"".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: None,
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_combined_filters() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    // Create repository matching both tag and name filters
    let matching_repo_dir = temp_dir.path().join("matching-repo");
    fs::create_dir_all(&matching_repo_dir).unwrap();
    create_git_repo(&matching_repo_dir).unwrap();

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
    create_git_repo(&wrong_name_repo_dir).unwrap();

    let wrong_name_repo = Repository {
        name: "wrong-name-repo".to_string(),
        url: "https://github.com/user/wrong-name-repo.git".to_string(),
        tags: vec!["backend".to_string()],
        path: Some(wrong_name_repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo combined".to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![matching_repo, wrong_name_repo],
        },
        tag: Some("backend".to_string()),
        repos: Some(vec!["matching-repo".to_string()]),
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}
