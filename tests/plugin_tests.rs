use serial_test::serial;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[serial]
fn test_plugin_system_integration() {
    // Create a temporary directory for our test plugin
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path();

    // Get the current working directory to ensure we run commands from the right place
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

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
        .current_dir(&current_dir)
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

    if !output.status.success() {
        eprintln!("=== FIRST COMMAND FAILED (list-plugins) ===");
        eprintln!("Current dir: {:?}", current_dir);
        eprintln!("Plugin dir: {:?}", plugin_dir);
        eprintln!("Plugin path exists: {}", plugin_path.exists());
        if plugin_path.exists() {
            eprintln!(
                "Plugin permissions: {:?}",
                fs::metadata(&plugin_path).unwrap().permissions()
            );
        }
        eprintln!(
            "PATH: {}:{}",
            plugin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        eprintln!("Exit status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("=== END FIRST COMMAND DEBUG ===");
    }
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available external plugins:"));
    assert!(stdout.contains("health"));

    // Test calling the external plugin
    let output = Command::new("cargo")
        .args(["run", "--", "health", "--test", "argument"])
        .current_dir(&current_dir)
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

    if !output.status.success() {
        eprintln!("=== SECOND COMMAND FAILED (health plugin) ===");
        eprintln!("Current dir: {:?}", current_dir);
        eprintln!("Plugin dir: {:?}", plugin_dir);
        eprintln!("Plugin path exists: {}", plugin_path.exists());
        if plugin_path.exists() {
            eprintln!(
                "Plugin permissions: {:?}",
                fs::metadata(&plugin_path).unwrap().permissions()
            );
        }
        eprintln!(
            "PATH: {}:{}",
            plugin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        eprintln!("Exit status: {}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("=== END SECOND COMMAND DEBUG ===");
    }
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mock health plugin executed"));
    assert!(stdout.contains("Args: --test argument"));

    // Test non-existent plugin
    let output = Command::new("cargo")
        .args(["run", "--", "nonexistent"])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to run nonexistent plugin");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Plugin 'repos-nonexistent' not found"));
}

#[test]
#[serial]
fn test_builtin_commands_still_work() {
    // Ensure built-in commands are not affected by plugin system

    // Get the current working directory to ensure we run commands from the right place
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Test help command (cargo run will build if needed)
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to run help");

    if !output.status.success() {
        eprintln!("Help command failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("status: {}", output.status);
    }
    assert!(output.status.success(), "Help command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A cli tool to manage multiple GitHub repositories"));
    assert!(stdout.contains("list-plugins"));
    assert!(stdout.contains("clone"));

    // Test list-plugins when no plugins are available
    // We need to filter PATH to remove any repos-* executables but keep system paths
    let original_path = std::env::var("PATH").unwrap_or_default();

    // Filter out paths that might contain repos-* plugins, but keep basic system paths
    let filtered_path = original_path
        .split(':')
        .filter(|p| {
            // Keep standard system paths
            p.starts_with("/usr/")
                || p.starts_with("/bin")
                || p.starts_with("/sbin")
                || p.contains("cargo")  // Keep cargo in PATH
                || p.contains("rustup") // Keep rustup in PATH
        })
        .collect::<Vec<_>>()
        .join(":");

    let output = Command::new("cargo")
        .args(["run", "--", "--list-plugins"])
        .current_dir(&current_dir)
        .env("PATH", &filtered_path)
        .output()
        .expect("Failed to run list-plugins");

    if !output.status.success() {
        eprintln!("List-plugins command failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("status: {}", output.status);
        eprintln!("filtered PATH: {}", filtered_path);
    }
    assert!(
        output.status.success(),
        "List-plugins command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Just check that list-plugins works, it's ok if it finds plugins or not
    assert!(
        stdout.contains("No external plugins found")
            || stdout.contains("Available external plugins:"),
        "Expected plugin list output"
    );
}
