use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::Command;

/// Prefix for external plugin executables
const PLUGIN_PREFIX: &str = "repos-";

/// Try to execute an external plugin
pub fn try_external_plugin(plugin_name: &str, args: &[String]) -> Result<()> {
    let binary_name = format!("{}{}", PLUGIN_PREFIX, plugin_name);

    let mut cmd = Command::new(&binary_name);
    cmd.args(args);

    let status = cmd.status().map_err(|e| {
        anyhow::anyhow!(
            "Plugin '{}' not found or failed to execute: {}",
            binary_name,
            e
        )
    })?;

    if !status.success() {
        anyhow::bail!("Plugin '{}' exited with status: {}", binary_name, status);
    }

    Ok(())
}

/// List all available external plugins by scanning PATH
pub fn list_external_plugins() -> Vec<String> {
    let mut plugins = Vec::new();

    if let Ok(path_env) = env::var("PATH") {
        for path_dir in env::split_paths(&path_env) {
            if let Ok(entries) = std::fs::read_dir(&path_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str()
                        && file_name.starts_with(PLUGIN_PREFIX)
                        && is_executable(&entry.path())
                        && let Some(plugin_name) = file_name.strip_prefix(PLUGIN_PREFIX)
                        && !plugin_name.is_empty()
                        && !plugins.contains(&plugin_name.to_string())
                    {
                        plugins.push(plugin_name.to_string());
                    }
                }
            }
        }
    }

    plugins.sort();
    plugins
}

/// Check if a file is executable
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        // On Windows, check if file has executable extension
        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
            let executable_extensions = ["exe", "bat", "cmd", "com"];
            return executable_extensions
                .iter()
                .any(|&ext| ext.eq_ignore_ascii_case(extension));
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_list_external_plugins_empty() {
        // Test with empty PATH
        let original_path = env::var("PATH").ok();
        unsafe {
            env::set_var("PATH", "");
        }

        let plugins = list_external_plugins();
        assert!(plugins.is_empty());

        // Restore original PATH
        if let Some(path) = original_path {
            unsafe {
                env::set_var("PATH", path);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_list_external_plugins_with_mock_plugins() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // Create mock plugin files
        let plugin1_path = plugin_dir.join("repos-health");
        let plugin2_path = plugin_dir.join("repos-security");
        let non_plugin_path = plugin_dir.join("other-tool");
        let non_executable_path = plugin_dir.join("repos-nonexec");

        fs::write(&plugin1_path, "#!/bin/sh\necho 'health plugin'").unwrap();
        fs::write(&plugin2_path, "#!/bin/sh\necho 'security plugin'").unwrap();
        fs::write(&non_plugin_path, "#!/bin/sh\necho 'not a plugin'").unwrap();
        fs::write(&non_executable_path, "echo 'not executable'").unwrap();

        // Make plugins executable
        let mut perms = fs::metadata(&plugin1_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&plugin1_path, perms).unwrap();

        let mut perms = fs::metadata(&plugin2_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&plugin2_path, perms).unwrap();

        let mut perms = fs::metadata(&non_plugin_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&non_plugin_path, perms).unwrap();

        // Update PATH to include our temp directory
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", plugin_dir.display(), original_path);
        unsafe {
            env::set_var("PATH", &new_path);
        }

        let plugins = list_external_plugins();

        // Should find health and security plugins, but not the others
        assert!(plugins.contains(&"health".to_string()));
        assert!(plugins.contains(&"security".to_string()));
        assert!(!plugins.contains(&"other-tool".to_string()));
        assert!(!plugins.contains(&"nonexec".to_string()));

        // Restore original PATH
        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn test_is_executable() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file");
        fs::write(&file_path, "test content").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Initially not executable
            assert!(!is_executable(&file_path));

            // Make executable
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&file_path, perms).unwrap();

            assert!(is_executable(&file_path));
        }

        #[cfg(windows)]
        {
            // Test with .exe extension
            let exe_path = temp_dir.path().join("test.exe");
            fs::write(&exe_path, "test content").unwrap();
            assert!(is_executable(&exe_path));

            // Test with .bat extension
            let bat_path = temp_dir.path().join("test.bat");
            fs::write(&bat_path, "test content").unwrap();
            assert!(is_executable(&bat_path));
        }
    }
}
