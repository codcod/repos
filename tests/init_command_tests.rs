use repos::{
    commands::{Command, CommandContext, init::InitCommand},
    config::Config,
};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_init_command_no_repositories_found() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to temp directory (empty, no git repos)
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let output_path = temp_dir.path().join("empty-config.yaml");
    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false,
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
    assert!(result.is_ok()); // Should succeed but not create file

    // Verify no config file was created
    assert!(!output_path.exists());

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_init_command_no_overwrite_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("existing-config.yaml");

    // Create existing file
    fs::write(&output_path, "existing content").unwrap();

    let command = InitCommand {
        output: output_path.to_string_lossy().to_string(),
        overwrite: false, // Should not overwrite
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
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    // Verify file was not modified
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "existing content");
}

#[tokio::test]
async fn test_init_command_structure() {
    // Test that we can create the command and it has the right fields
    let command = InitCommand {
        output: "test.yaml".to_string(),
        overwrite: true,
    };

    assert_eq!(command.output, "test.yaml");
    assert!(command.overwrite);
}
