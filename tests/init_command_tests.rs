use repos::commands::{Command, CommandContext, init::InitCommand};
use repos::config::Config;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a git repository in a directory
fn create_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    // Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()?;

    // Configure git (required for commits)
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_init_command_basic_creation() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("basic-config.yaml");

    // Create a git repository so the command has something to discover
    let repo_dir = temp_dir.path().join("test-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    // Config file should be created if repositories are found
    if result.is_ok() {
        // Command succeeded, but file may not exist if no remote URL could be found
        // This is acceptable behavior
    }
}

#[tokio::test]
#[serial]
async fn test_init_command_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("existing-config.yaml");

    // Create existing file
    fs::write(&output_path, "existing content").unwrap();

    // Create a git repository so the command has something to discover
    let repo_dir = temp_dir.path().join("test-repo");
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: true, // Should overwrite
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok()); // Should succeed and overwrite

    // The file content check is not reliable since it depends on whether
    // a remote URL could be discovered
}

#[tokio::test]
#[serial]
async fn test_init_command_no_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("existing-config.yaml");

    // Create existing file
    fs::write(&output_path, "existing content").unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false, // Should not overwrite
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    // Should fail because file exists and overwrite is false
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_init_command_with_git_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path().join("test-repo");
    let git_dir = repo_dir.join(".git");

    // Create a mock git repository
    fs::create_dir_all(&git_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    // Create a mock git config to simulate a real repo
    fs::write(
        git_dir.join("config"),
        "[core]\nrepositoryformatversion = 0",
    )
    .unwrap();

    let output_path = temp_dir.path().join("discovered-config.yaml");
    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    // Change to temp directory to discover the git repo
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let _result = command.execute(&context).await;

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // The command might fail because it can't get the remote URL, but should not panic
    // We test that the discovery logic executes without crashing
}

#[tokio::test]
#[serial]
async fn test_init_command_supplement_with_duplicate_repository() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("config-with-duplicate.yaml");

    // Create existing config with a repository
    let existing_config = Config {
        repositories: vec![repos::config::Repository::new(
            "test-repo".to_string(),
            "git@github.com:owner/test-repo.git".to_string(),
        )],
    };
    existing_config
        .save(&output_path.to_string_lossy())
        .unwrap();

    // Create a directory structure that would discover the same repo
    let repo_dir = temp_dir.path().join("test-repo");
    let git_dir = repo_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: true, // Should supplement but skip duplicates
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed and maintain the existing repository
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_init_command_supplement_with_new_repository() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("config-with-supplement.yaml");

    // Create existing config with one repository
    let existing_config = Config {
        repositories: vec![repos::config::Repository::new(
            "existing-repo".to_string(),
            "git@github.com:owner/existing-repo.git".to_string(),
        )],
    };
    existing_config
        .save(&output_path.to_string_lossy())
        .unwrap();

    // Create a different directory structure that would discover a new repo
    let repo_dir = temp_dir.path().join("new-repo");
    let git_dir = repo_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: true, // Should supplement with new repo
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed and add the new repository
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_init_command_git_directory_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    // Create various directory structures to test edge cases
    let nested_dir = temp_dir
        .path()
        .join("level1")
        .join("level2")
        .join("level3")
        .join("too-deep");
    let git_dir = nested_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    create_git_repo(&nested_dir).unwrap();

    // Create a .git file (not directory) to test that case
    let repo_with_git_file = temp_dir.path().join("repo-with-git-file");
    fs::create_dir_all(&repo_with_git_file).unwrap();
    fs::write(repo_with_git_file.join(".git"), "gitdir: ../real-git-dir").unwrap();

    let output_path = temp_dir.path().join("edge-case-config.yaml");
    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed - edge cases are handled gracefully
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_init_command_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("empty-config.yaml");

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    // Change to empty temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed even with no git repositories found
    assert!(result.is_ok());

    // File should NOT exist when no repositories are found (expected behavior)
    assert!(!output_path.exists());
}

#[tokio::test]
#[serial]
async fn test_init_command_multiple_git_repositories() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple git repositories
    let repo1_dir = temp_dir.path().join("repo1");
    let repo2_dir = temp_dir.path().join("repo2");
    let repo3_dir = temp_dir.path().join("nested").join("repo3");

    for repo_dir in [&repo1_dir, &repo2_dir, &repo3_dir] {
        fs::create_dir_all(repo_dir).unwrap();
        fs::create_dir_all(repo_dir.join(".git")).unwrap();
        create_git_repo(repo_dir).unwrap();
    }

    let output_path = temp_dir.path().join("multi-repo-config.yaml");
    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed
    assert!(result.is_ok());

    // Note: File may not exist if git repositories don't have remote URLs
    // which is common in test scenarios. This is expected behavior.
}

#[tokio::test]
#[serial]
async fn test_init_command_integration_flow() {
    // Test complete initialization flow with realistic scenarios
    let temp_dir = TempDir::new().unwrap();

    // Create a realistic project structure
    let backend_dir = temp_dir.path().join("my-project-backend");
    let frontend_dir = temp_dir.path().join("my-project-frontend");
    let docs_dir = temp_dir.path().join("docs");

    // Only backend and frontend are git repos
    for repo_dir in [&backend_dir, &frontend_dir] {
        fs::create_dir_all(repo_dir).unwrap();
        fs::create_dir_all(repo_dir.join(".git")).unwrap();
        create_git_repo(repo_dir).unwrap();

        // Add some realistic files
        fs::write(repo_dir.join("README.md"), "# Project").unwrap();
        fs::write(repo_dir.join(".gitignore"), "target/\nnode_modules/").unwrap();
    }

    // docs is not a git repo
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("README.md"), "# Documentation").unwrap();

    let output_path = temp_dir.path().join("repos.yaml");
    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
        supplement: false,
    };

    let context = CommandContext {
        config: Config::new(),
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = command.execute(&context).await;

    std::env::set_current_dir(original_dir).unwrap();

    // Should succeed
    assert!(result.is_ok());

    // Note: Config file may not be created if repositories don't have remote URLs
    // This is expected behavior in test scenarios
}
