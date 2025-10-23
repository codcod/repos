use repos::{
    commands::{Command, CommandContext, run::RunCommand},
    config::{Config, Repository},
};
use std::fs;
use std::path::PathBuf;
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
async fn test_run_command_basic_creation() {
    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: false,
        output_dir: None,
    };

    assert_eq!(command.command, "echo hello");
    assert!(!command.no_save);
    assert!(command.output_dir.is_none());
}

#[tokio::test]
async fn test_run_command_with_custom_output_dir() {
    let output_dir = PathBuf::from("/tmp/custom");
    let command = RunCommand {
        command: "ls".to_string(),
        no_save: false,
        output_dir: Some(output_dir.clone()),
    };

    assert_eq!(command.command, "ls");
    assert!(!command.no_save);
    assert_eq!(command.output_dir, Some(output_dir));
}

#[tokio::test]
async fn test_run_command_no_save_mode() {
    let command = RunCommand {
        command: "pwd".to_string(),
        no_save: true,
        output_dir: None,
    };

    assert_eq!(command.command, "pwd");
    assert!(command.no_save);
    assert!(command.output_dir.is_none());
}

#[tokio::test]
async fn test_run_command_empty_repositories() {
    let command = RunCommand {
        command: "echo test".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config::new(), // Empty config
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok()); // Should succeed with empty repos
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
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_no_matching_repos() {
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
        no_save: true,
        output_dir: None,
    };

    // Use a tag that doesn't match any repo
    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: vec!["nonexistent".to_string()],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_specific_repos() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let repo_dir1 = temp_dir.path().join("test-repo1");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    let repo_dir2 = temp_dir.path().join("test-repo2");
    fs::create_dir_all(&repo_dir2).unwrap();
    create_git_repo(&repo_dir2).unwrap();

    let repo1 = Repository {
        name: "test-repo1".to_string(),
        url: "https://github.com/user/test-repo1.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir1.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let repo2 = Repository {
        name: "test-repo2".to_string(),
        url: "https://github.com/user/test-repo2.git".to_string(),
        tags: vec!["other".to_string()],
        path: Some(repo_dir2.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo1, repo2],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: Some(vec!["test-repo1".to_string()]),
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_tag_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let repo_dir1 = temp_dir.path().join("backend-repo");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    let repo_dir2 = temp_dir.path().join("frontend-repo");
    fs::create_dir_all(&repo_dir2).unwrap();
    create_git_repo(&repo_dir2).unwrap();

    let backend_repo = Repository {
        name: "backend-repo".to_string(),
        url: "https://github.com/user/backend-repo.git".to_string(),
        tags: vec!["backend".to_string(), "rust".to_string()],
        path: Some(repo_dir1.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let frontend_repo = Repository {
        name: "frontend-repo".to_string(),
        url: "https://github.com/user/frontend-repo.git".to_string(),
        tags: vec!["frontend".to_string(), "javascript".to_string()],
        path: Some(repo_dir2.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![backend_repo, frontend_repo],
        },
        tag: vec!["backend".to_string()],
        exclude_tag: vec![],
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

    let repo_dir1 = temp_dir.path().join("test-repo1");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    let repo_dir2 = temp_dir.path().join("test-repo2");
    fs::create_dir_all(&repo_dir2).unwrap();
    create_git_repo(&repo_dir2).unwrap();

    let repo1 = Repository {
        name: "test-repo1".to_string(),
        url: "https://github.com/user/test-repo1.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir1.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let repo2 = Repository {
        name: "test-repo2".to_string(),
        url: "https://github.com/user/test-repo2.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir2.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo1, repo2],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: true,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_tag_filter_no_match() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let repo_dir = temp_dir.path().join("backend-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let backend_repo = Repository {
        name: "backend-repo".to_string(),
        url: "https://github.com/user/backend-repo.git".to_string(),
        tags: vec!["backend".to_string(), "rust".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![backend_repo],
        },
        tag: vec!["frontend".to_string()], // Non-matching tag
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

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
        command: "false".to_string(), // Command that will fail
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    // The command should fail when all individual commands fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_command_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

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

    // Test command that creates a file
    let command = RunCommand {
        command: "touch test-file.txt".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Verify the file was created
    assert!(repo_dir.join("test-file.txt").exists());
}

#[tokio::test]
async fn test_run_command_with_multiple_tags() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let repo_dir = temp_dir.path().join("backend-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let backend_repo = Repository {
        name: "backend-repo".to_string(),
        url: "https://github.com/user/backend-repo.git".to_string(),
        tags: vec!["backend".to_string(), "rust".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    // Test with multiple matching tags
    let context = CommandContext {
        config: Config {
            repositories: vec![backend_repo],
        },
        tag: vec!["backend".to_string()],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_special_characters() {
    let command = RunCommand {
        command: "echo \"test with spaces and symbols: @#$%\"".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_parallel_mode() {
    let command = RunCommand {
        command: "echo parallel test".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        parallel: true, // Test parallel execution
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_comprehensive() {
    // Test all options together with real git repositories
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test repos
    let repo_dir1 = temp_dir.path().join("comprehensive-repo1");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    let repo_dir2 = temp_dir.path().join("comprehensive-repo2");
    fs::create_dir_all(&repo_dir2).unwrap();
    create_git_repo(&repo_dir2).unwrap();

    let command = RunCommand {
        command: "echo comprehensive test".to_string(),
        no_save: false,
        output_dir: Some(temp_dir.path().to_path_buf()),
    };

    let config = Config {
        repositories: vec![
            Repository {
                name: "comprehensive-repo1".to_string(),
                url: "https://github.com/test/comprehensive1.git".to_string(),
                tags: vec!["backend".to_string()],
                path: Some(repo_dir1.to_string_lossy().to_string()),
                branch: None,
                config_dir: None,
            },
            Repository {
                name: "comprehensive-repo2".to_string(),
                url: "https://github.com/test/comprehensive2.git".to_string(),
                tags: vec!["frontend".to_string()],
                path: Some(repo_dir2.to_string_lossy().to_string()),
                branch: None,
                config_dir: None,
            },
        ],
    };

    let context = CommandContext {
        config,
        tag: vec!["backend".to_string()], // Should filter to repo1 only
        exclude_tag: vec![],
        parallel: true,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_exclude_tag_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_dir = temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();

    let repo_dir1 = temp_dir.path().join("backend-repo");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    let repo_dir2 = temp_dir.path().join("frontend-repo");
    fs::create_dir_all(&repo_dir2).unwrap();
    create_git_repo(&repo_dir2).unwrap();

    let backend_repo = Repository {
        name: "backend-repo".to_string(),
        url: "https://github.com/user/backend-repo.git".to_string(),
        tags: vec!["backend".to_string(), "rust".to_string()],
        path: Some(repo_dir1.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let frontend_repo = Repository {
        name: "frontend-repo".to_string(),
        url: "https://github.com/user/frontend-repo.git".to_string(),
        tags: vec!["frontend".to_string(), "javascript".to_string()],
        path: Some(repo_dir2.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        command: "echo hello".to_string(),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![backend_repo, frontend_repo],
        },
        tag: vec![],
        exclude_tag: vec!["frontend".to_string()], // Should exclude frontend repo
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}
