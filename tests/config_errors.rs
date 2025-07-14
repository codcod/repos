//! Config file error scenario tests

use repos::config::Config;
use std::fs;

#[test]
fn test_config_file_not_found() {
    let result = Config::load("nonexistent_config.yaml");

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No such file") || error.to_string().contains("not found"));
}

#[test]
fn test_config_file_invalid_yaml() {
    let invalid_yaml = "invalid: yaml: content: [unclosed";
    fs::write("invalid.yaml", invalid_yaml).expect("Failed to write test file");

    let result = Config::load("invalid.yaml");

    // Clean up
    fs::remove_file("invalid.yaml").ok();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("mapping values are not allowed")
            || error.to_string().contains("yaml")
            || error.to_string().contains("parse")
            || error.to_string().contains("deserialize")
            || error.to_string().contains("EOF")
            || error.to_string().contains("expected")
    );
}

#[test]
fn test_config_file_missing_repositories_field() {
    let yaml_content = r#"
some_other_field: value
"#;
    fs::write("missing_repos.yaml", yaml_content).expect("Failed to write test file");

    let result = Config::load("missing_repos.yaml");

    // Clean up
    fs::remove_file("missing_repos.yaml").ok();

    assert!(result.is_err());
}

#[test]
fn test_config_file_invalid_repository_structure() {
    let yaml_content = r#"
repositories:
  - invalid_repo_without_required_fields: true
"#;
    fs::write("invalid_repo.yaml", yaml_content).expect("Failed to write test file");

    let result = Config::load("invalid_repo.yaml");

    // Clean up
    fs::remove_file("invalid_repo.yaml").ok();

    assert!(result.is_err());
}

#[test]
fn test_config_file_empty() {
    fs::write("empty.yaml", "").expect("Failed to write test file");

    let result = Config::load("empty.yaml");

    // Clean up
    fs::remove_file("empty.yaml").ok();

    assert!(result.is_err());
}

#[test]
fn test_config_file_permission_denied() {
    // Skip this test on Windows as permission handling is different
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Create a file and remove read permissions
        fs::write("no_permission.yaml", "repositories: []").expect("Failed to write test file");

        let mut perms = fs::metadata("no_permission.yaml").unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions("no_permission.yaml", perms).ok();

        let result = Config::load("no_permission.yaml");

        // Clean up
        let mut perms = fs::metadata("no_permission.yaml").unwrap().permissions();
        perms.set_mode(0o644); // Restore permissions for cleanup
        fs::set_permissions("no_permission.yaml", perms).ok();
        fs::remove_file("no_permission.yaml").ok();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("permission") || error.to_string().contains("denied"));
    }
}

#[test]
fn test_config_directory_instead_of_file() {
    // Create a directory with the config name
    fs::create_dir("config_dir.yaml").expect("Failed to create directory");

    let result = Config::load("config_dir.yaml");

    // Clean up
    fs::remove_dir("config_dir.yaml").ok();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("directory")
            || error.to_string().contains("Is a directory")
            || error.to_string().contains("invalid")
    );
}
