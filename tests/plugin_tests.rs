use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_plugin_system_integration() {
    // Create a temporary directory for our test plugin
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // Create a mock plugin
    let plugin_content = r#"#!/bin/bash
echo "Mock health plugin executed"
echo "Args: $@"
exit 0
"#;

    let plugin_path = plugin_dir.join("repos-health");
    fs::write(&plugin_path, plugin_content).unwrap();

    // Make it executable
    let mut perms = fs::metadata(&plugin_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&plugin_path, perms).unwrap();

    // Test list-plugins with our mock plugin
    let output = Command::new("cargo")
        .args(["run", "--", "--list-plugins"])
        .env(
            "PATH",
            format!(
                "{}:{}",
                plugin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("Failed to run list-plugins");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available external plugins:"));
    assert!(stdout.contains("health"));

    // Test calling the external plugin
    let output = Command::new("cargo")
        .args(["run", "--", "health", "--test", "argument"])
        .env(
            "PATH",
            format!(
                "{}:{}",
                plugin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("Failed to run health plugin");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mock health plugin executed"));
    assert!(stdout.contains("Args: --test argument"));

    // Test non-existent plugin
    let output = Command::new("cargo")
        .args(["run", "--", "nonexistent"])
        .output()
        .expect("Failed to run nonexistent plugin");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Plugin 'repos-nonexistent' not found"));
}

#[test]
fn test_builtin_commands_still_work() {
    // Ensure built-in commands are not affected by plugin system

    // Test help command
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A cli tool to manage multiple GitHub repositories"));
    assert!(stdout.contains("list-plugins"));
    assert!(stdout.contains("clone"));

    // Test list-plugins when no plugins are available
    let temp_empty_dir = TempDir::new().unwrap();
    let output = Command::new("cargo")
        .args(["run", "--", "--list-plugins"])
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp_empty_dir.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("Failed to run list-plugins");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No external plugins found"));
}
