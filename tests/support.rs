//! Common test support utilities and fixtures
//!
//! This module provides shared functionality to reduce code duplication
//! across integration and E2E tests.

use repos::{
    commands::CommandContext,
    config::{Config, Recipe, Repository},
};
use std::{fs, path::PathBuf, process::Command};
use tempfile::TempDir;

/// Result of running a CLI command
#[derive(Debug)]
pub struct CliOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// A test workspace with temporary directory and config management
pub struct Workspace {
    pub root: TempDir,
    pub config_path: PathBuf,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    /// Create a new temporary workspace
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp directory");
        let config_path = root.path().join("config.yaml");
        Self { root, config_path }
    }

    /// Write configuration YAML to the workspace
    pub fn write_config(&self, yaml: &str) {
        fs::write(&self.config_path, yaml).expect("Failed to write config");
    }

    /// Get the workspace root path
    pub fn path(&self) -> &std::path::Path {
        self.root.path()
    }

    /// Get the config file path as string
    pub fn config_str(&self) -> &str {
        self.config_path.to_str().expect("Config path not UTF-8")
    }
}

/// Run the repos CLI with given arguments
pub fn run_cli(args: &[&str], cwd: Option<&std::path::Path>) -> CliOutput {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output().expect("Failed to execute cargo run");

    CliOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Initialize a git repository with basic configuration
pub fn init_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;

    // Initialize git repo
    Command::new("git").arg("init").current_dir(path).output()?;

    // Configure git (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    // Create a file and commit
    fs::write(path.join("README.md"), "# Test Repository")?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

/// Create a repository instance for testing
pub fn create_test_repo(name: &str, temp_dir: &TempDir) -> Repository {
    let repo_dir = temp_dir.path().join(name);
    init_git_repo(&repo_dir).expect("Failed to initialize git repo");

    Repository {
        name: name.to_string(),
        url: format!("https://github.com/user/{}.git", name),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    }
}

/// Create a recipe for testing
pub fn create_test_recipe(name: &str, steps: Vec<&str>) -> Recipe {
    Recipe {
        name: name.to_string(),
        steps: steps.into_iter().map(|s| s.to_string()).collect(),
    }
}

/// Create a test CommandContext with given repositories and recipes
pub fn create_test_context(repositories: Vec<Repository>, recipes: Vec<Recipe>) -> CommandContext {
    CommandContext {
        config: Config {
            repositories,
            recipes,
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    }
}

/// Read metadata.json from a directory
pub fn read_metadata(dir: &std::path::Path) -> serde_json::Value {
    let data = fs::read(dir.join("metadata.json")).expect("Failed to read metadata.json");
    serde_json::from_slice(&data).expect("Failed to parse metadata.json")
}

/// Assert that metadata contains required command fields
pub fn assert_command_metadata(json: &serde_json::Value) {
    let obj = json.as_object().expect("Metadata should be an object");
    assert!(obj.get("command").is_some(), "Missing 'command' field");
    assert!(
        obj.get("recipe").is_none(),
        "Should not have 'recipe' field for commands"
    );
    assert!(obj.get("exit_code").is_some(), "Missing 'exit_code' field");
    assert!(
        obj.get("exit_code_description").is_some(),
        "Missing 'exit_code_description' field"
    );
    assert!(
        obj.get("start_time").is_some(),
        "Missing 'start_time' field"
    );
    assert!(obj.get("end_time").is_some(), "Missing 'end_time' field");
}

/// Assert that metadata contains required recipe fields
pub fn assert_recipe_metadata(json: &serde_json::Value) {
    let obj = json.as_object().expect("Metadata should be an object");
    assert!(obj.get("recipe").is_some(), "Missing 'recipe' field");
    assert!(
        obj.get("command").is_none(),
        "Should not have 'command' field for recipes"
    );
    assert!(
        obj.get("recipe_steps").is_some(),
        "Missing 'recipe_steps' field"
    );
    assert!(obj.get("exit_code").is_some(), "Missing 'exit_code' field");
    assert!(
        obj.get("exit_code_description").is_some(),
        "Missing 'exit_code_description' field"
    );
    assert!(
        obj.get("start_time").is_some(),
        "Missing 'start_time' field"
    );
    assert!(obj.get("end_time").is_some(), "Missing 'end_time' field");
}

/// Check if timestamp has expected format (YYYY-MM-DD HH:MM:SS)
pub fn is_valid_timestamp_format(ts: &str) -> bool {
    let parts: Vec<_> = ts.split(' ').collect();
    if parts.len() != 2 {
        return false;
    }

    let date = parts[0];
    let time = parts[1];

    date.len() == 10
        && time.len() == 8
        && date.chars().nth(4) == Some('-')
        && date.chars().nth(7) == Some('-')
        && time.chars().nth(2) == Some(':')
        && time.chars().nth(5) == Some(':')
}

/// Assert that timestamps in metadata are valid
pub fn assert_valid_timestamps(json: &serde_json::Value) {
    let obj = json.as_object().expect("Metadata should be an object");

    if let Some(start_time) = obj.get("start_time").and_then(|v| v.as_str()) {
        assert!(
            is_valid_timestamp_format(start_time),
            "Invalid start_time format: {}",
            start_time
        );
    }

    if let Some(end_time) = obj.get("end_time").and_then(|v| v.as_str()) {
        assert!(
            is_valid_timestamp_format(end_time),
            "Invalid end_time format: {}",
            end_time
        );
    }
}

/// Create a script file with given content and make it executable
#[cfg(unix)]
pub fn create_executable_script(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::write(path, content)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

/// Create a script file with given content (Windows compatible)
#[cfg(not(unix))]
pub fn create_executable_script(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

/// Exit code mapping for assertions (extracted for unit testing)
pub fn map_exit_code(code: i32) -> &'static str {
    match code {
        0 => "success",
        1 => "general error",
        2 => "misuse of shell builtins",
        126 => "command invoked cannot execute",
        127 => "command not found",
        130 => "script terminated by Control-C",
        c if c > 128 => "terminated by signal",
        _ => "error",
    }
}

/// Sanitize a name for safe filesystem usage (extracted for unit testing)
pub fn sanitize_name(input: &str) -> String {
    let mut result = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    if result.len() > 50 {
        result.truncate(50);
    }

    if result.is_empty() {
        result = "unnamed".to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_exit_code() {
        assert_eq!(map_exit_code(0), "success");
        assert_eq!(map_exit_code(1), "general error");
        assert_eq!(map_exit_code(2), "misuse of shell builtins");
        assert_eq!(map_exit_code(126), "command invoked cannot execute");
        assert_eq!(map_exit_code(127), "command not found");
        assert_eq!(map_exit_code(130), "script terminated by Control-C");
        assert_eq!(map_exit_code(131), "terminated by signal");
        assert_eq!(map_exit_code(255), "terminated by signal");
        assert_eq!(map_exit_code(42), "error");
        assert_eq!(map_exit_code(-1), "error");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("hello-world"), "hello-world");
        assert_eq!(sanitize_name("hello world"), "hello_world");
        assert_eq!(sanitize_name("test@example.com"), "test_example_com");
        assert_eq!(sanitize_name(""), "unnamed");

        let long_name = "a".repeat(60);
        let sanitized = sanitize_name(&long_name);
        assert_eq!(sanitized.len(), 50);
        assert_eq!(sanitized, "a".repeat(50));
    }

    #[test]
    fn test_timestamp_format_validation() {
        assert!(is_valid_timestamp_format("2023-10-25 14:30:45"));
        assert!(is_valid_timestamp_format("1999-01-01 00:00:00"));
        assert!(!is_valid_timestamp_format("2023-10-25"));
        assert!(!is_valid_timestamp_format("14:30:45"));
        assert!(!is_valid_timestamp_format("2023-10-25T14:30:45"));
        assert!(!is_valid_timestamp_format("2023/10/25 14:30:45"));
        assert!(!is_valid_timestamp_format(""));
    }
}
