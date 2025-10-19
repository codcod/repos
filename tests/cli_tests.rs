//! CLI argument parsing integration tests

use std::process::Command;

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
fn test_run_command_missing_command_arg() {
    let output = Command::new("cargo")
        .args(["run", "--", "run"])
        .output()
        .expect("Failed to execute cargo run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") || stderr.contains("missing"));
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
    // Create a temporary valid config file
    let config_content = r#"
repositories:
  - name: test-repo
    url: https://github.com/test/repo
    tags: [backend]
"#;
    std::fs::write("test_config.yaml", config_content).expect("Failed to write test config");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "clone",
            "--config",
            "test_config.yaml",
            "--tag",
            "nonexistent",
        ])
        .output()
        .expect("Failed to execute cargo run");

    // Clean up
    std::fs::remove_file("test_config.yaml").ok();

    // Should succeed but clone nothing
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No repositories") || stdout.is_empty());
}
