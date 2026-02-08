//! CLI argument parsing integration tests

use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper struct for creating temporary test workspaces
struct Workspace {
    #[allow(dead_code)]
    root: TempDir,
    config_path: PathBuf,
}

impl Workspace {
    fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp dir");
        let config_path = root.path().join("repos.yaml");

        Self { root, config_path }
    }

    fn write_config(&self, content: &str) {
        std::fs::write(&self.config_path, content).expect("Failed to write config");
    }

    fn config_str(&self) -> &str {
        self.config_path.to_str().expect("Invalid config path")
    }
}

/// Result of running a CLI command
#[derive(Debug)]
struct CliOutput {
    status: i32,
    stdout: String,
    stderr: String,
}

/// Run the repos CLI with given arguments
fn run_cli(args: &[&str]) -> CliOutput {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args(args);

    // Always run cargo from the project root
    let project_root = env::current_dir().expect("Failed to get current directory");
    cmd.current_dir(&project_root);

    let output = cmd.output().expect("Failed to execute cargo run");

    CliOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
fn test_cli_missing_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn test_cli_invalid_subcommand() {
    let output = Command::new("cargo")
        .args(["run", "--", "invalid-command"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand") || stderr.contains("invalid"));
}

#[test]
fn test_clone_command_missing_config() {
    let output = Command::new("cargo")
        .args(["run", "--", "clone", "--config", "nonexistent.yaml"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such file") || stderr.contains("not found"));
}

#[test]
fn test_run_command_missing_command_and_recipe() {
    let ws = Workspace::new();
    ws.write_config(
        r#"
repositories:
  - name: test-repo
    url: https://github.com/test/repo
    tags: [test]
"#,
    );

    let output = run_cli(&["run", "--config", ws.config_str()]);

    assert_ne!(output.status, 0);
    assert!(
        output
            .stderr
            .contains("Either --recipe or a command must be provided")
    );
}

#[test]
fn test_run_command_both_command_and_recipe() {
    let ws = Workspace::new();
    ws.write_config(
        r#"
repositories:
  - name: test-repo
    url: https://github.com/test/repo
    tags: [test]
recipes:
  - name: test-recipe
    steps:
      - echo "test recipe"
"#,
    );

    let output = run_cli(&[
        "run",
        "--config",
        ws.config_str(),
        "--recipe",
        "test-recipe",
        "echo",
        "test",
    ]);

    assert_ne!(output.status, 0);
    assert!(
        output
            .stderr
            .contains("Cannot specify both command and --recipe")
    );
}

#[test]
fn test_pr_command_missing_required_args() {
    let output = Command::new("cargo")
        .args(["run", "--", "pr"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // PR command may have different required args, let's be more flexible
    assert!(
        stderr.contains("error")
            || stderr.contains("Error")
            || stderr.contains("required")
            || stderr.contains("missing")
            || stderr.contains("Usage")
    );
}

#[test]
fn test_remove_command_with_invalid_config() {
    let output = Command::new("cargo")
        .args(["run", "--", "rm", "test-repo", "--config", "invalid.yaml"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Error could be about file not found or other config issues
    assert!(
        stderr.contains("No such file") || stderr.contains("not found") || stderr.contains("error")
    );
}

#[test]
fn test_clone_with_invalid_tag() {
    let ws = Workspace::new();
    ws.write_config(
        r#"
repositories:
  - name: test-repo
    url: https://github.com/test/repo
    tags: [backend]
"#,
    );

    let output = run_cli(&["clone", "--config", ws.config_str(), "--tag", "nonexistent"]);

    // Should succeed but clone nothing
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("No repositories") || output.stdout.is_empty());
}
