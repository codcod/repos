use repos::util::{ensure_directory_exists, find_git_repositories};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to create a git repository in a directory
fn create_git_repo(path: &Path, remote_url: Option<&str>) -> std::io::Result<()> {
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

    // Add remote if provided
    if let Some(url) = remote_url {
        Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(path)
            .output()?;
    }

    Ok(())
}

#[test]
fn test_find_git_repositories_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert!(repos.is_empty());
}

#[test]
fn test_find_git_repositories_no_git_repos() {
    let temp_dir = TempDir::new().unwrap();

    // Create some non-git directories
    fs::create_dir_all(temp_dir.path().join("folder1")).unwrap();
    fs::create_dir_all(temp_dir.path().join("folder2")).unwrap();
    fs::write(temp_dir.path().join("folder1/file.txt"), "content").unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert!(repos.is_empty());
}

#[test]
fn test_find_git_repositories_single_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir_all(&repo_path).unwrap();

    create_git_repo(&repo_path, Some("https://github.com/user/test-repo.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "test-repo");
    assert_eq!(repos[0].url, "https://github.com/user/test-repo.git");
}

#[test]
fn test_find_git_repositories_multiple_repos() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple git repositories
    let repo1_path = temp_dir.path().join("repo1");
    let repo2_path = temp_dir.path().join("repo2");
    fs::create_dir_all(&repo1_path).unwrap();
    fs::create_dir_all(&repo2_path).unwrap();

    create_git_repo(&repo1_path, Some("https://github.com/user/repo1.git")).unwrap();
    create_git_repo(&repo2_path, Some("https://github.com/user/repo2.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 2);

    let repo_names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
    assert!(repo_names.contains(&"repo1"));
    assert!(repo_names.contains(&"repo2"));
}

#[test]
fn test_find_git_repositories_no_remote() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("local-repo");
    fs::create_dir_all(&repo_path).unwrap();

    // Create git repo without remote
    create_git_repo(&repo_path, None).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    // Should not include repos without remotes
    assert!(repos.is_empty());
}

#[test]
fn test_find_git_repositories_go_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("go-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create go.mod file
    fs::write(repo_path.join("go.mod"), "module test\n\ngo 1.19").unwrap();
    create_git_repo(&repo_path, Some("https://github.com/user/go-project.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"go".to_string()));
}

#[test]
fn test_find_git_repositories_javascript_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("js-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create package.json file
    fs::write(
        repo_path.join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .unwrap();
    create_git_repo(&repo_path, Some("https://github.com/user/js-project.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"javascript".to_string()));
    assert!(repos[0].tags.contains(&"node".to_string()));
}

#[test]
fn test_find_git_repositories_python_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("python-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create requirements.txt file
    fs::write(repo_path.join("requirements.txt"), "requests==2.28.1\n").unwrap();
    create_git_repo(
        &repo_path,
        Some("https://github.com/user/python-project.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"python".to_string()));
}

#[test]
fn test_find_git_repositories_java_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("java-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create pom.xml file
    fs::write(repo_path.join("pom.xml"), "<project></project>").unwrap();
    create_git_repo(&repo_path, Some("https://github.com/user/java-project.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"java".to_string()));
}

#[test]
fn test_find_git_repositories_rust_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("rust-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create Cargo.toml file
    fs::write(
        repo_path.join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    create_git_repo(&repo_path, Some("https://github.com/user/rust-project.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"rust".to_string()));
}

#[test]
fn test_find_git_repositories_frontend_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("frontend-app");
    fs::create_dir_all(&repo_path).unwrap();

    create_git_repo(&repo_path, Some("https://github.com/user/frontend-app.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"frontend".to_string()));
}

#[test]
fn test_find_git_repositories_backend_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("backend-api");
    fs::create_dir_all(&repo_path).unwrap();

    create_git_repo(&repo_path, Some("https://github.com/user/backend-api.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"backend".to_string()));
}

#[test]
fn test_find_git_repositories_mobile_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("mobile-app");
    fs::create_dir_all(&repo_path).unwrap();

    create_git_repo(&repo_path, Some("https://github.com/user/mobile-app.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"mobile".to_string()));
}

#[test]
fn test_find_git_repositories_multiple_file_types() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("fullstack-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create multiple language files
    fs::write(repo_path.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(repo_path.join("requirements.txt"), "django").unwrap();
    fs::write(repo_path.join("go.mod"), "module test").unwrap();

    create_git_repo(
        &repo_path,
        Some("https://github.com/user/fullstack-project.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);

    let tags = &repos[0].tags;
    assert!(tags.contains(&"javascript".to_string()));
    assert!(tags.contains(&"node".to_string()));
    assert!(tags.contains(&"python".to_string()));
    assert!(tags.contains(&"go".to_string()));
}

#[test]
fn test_find_git_repositories_depth_limit() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested directories beyond depth limit
    let deep_path = temp_dir
        .path()
        .join("level1")
        .join("level2")
        .join("level3")
        .join("level4")
        .join("deep-repo");
    fs::create_dir_all(&deep_path).unwrap();

    create_git_repo(&deep_path, Some("https://github.com/user/deep-repo.git")).unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    // Should not find repos beyond max_depth(3)
    assert!(repos.is_empty());
}

#[test]
fn test_find_git_repositories_within_depth_limit() {
    let temp_dir = TempDir::new().unwrap();

    // Create repo within depth limit
    let shallow_path = temp_dir
        .path()
        .join("level1")
        .join("level2")
        .join("shallow-repo");
    fs::create_dir_all(&shallow_path).unwrap();

    create_git_repo(
        &shallow_path,
        Some("https://github.com/user/shallow-repo.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "shallow-repo");
}

#[test]
fn test_ensure_directory_exists_new_directory() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new_directory");

    assert!(!new_dir.exists());
    ensure_directory_exists(new_dir.to_str().unwrap()).unwrap();
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_ensure_directory_exists_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let existing_dir = temp_dir.path().join("existing");
    fs::create_dir(&existing_dir).unwrap();

    assert!(existing_dir.exists());
    // Should not error on existing directory
    ensure_directory_exists(existing_dir.to_str().unwrap()).unwrap();
    assert!(existing_dir.exists());
}

#[test]
fn test_ensure_directory_exists_nested_path() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("level1").join("level2").join("level3");

    assert!(!nested_path.exists());
    ensure_directory_exists(nested_path.to_str().unwrap()).unwrap();
    assert!(nested_path.exists());
    assert!(nested_path.is_dir());

    // Check intermediate directories were created
    assert!(temp_dir.path().join("level1").exists());
    assert!(temp_dir.path().join("level1").join("level2").exists());
}

#[test]
fn test_find_git_repositories_invalid_path() {
    let result = find_git_repositories("/this/path/does/not/exist");
    // Should handle invalid paths gracefully
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_find_git_repositories_special_characters_in_name() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("repo-with-dashes_and_underscores");
    fs::create_dir_all(&repo_path).unwrap();

    create_git_repo(
        &repo_path,
        Some("https://github.com/user/repo-with-dashes_and_underscores.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "repo-with-dashes_and_underscores");
}

#[test]
fn test_find_git_repositories_pyproject_toml() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("modern-python");
    fs::create_dir_all(&repo_path).unwrap();

    // Create pyproject.toml file (modern Python project)
    fs::write(
        repo_path.join("pyproject.toml"),
        r#"[tool.poetry]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();
    create_git_repo(
        &repo_path,
        Some("https://github.com/user/modern-python.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"python".to_string()));
}

#[test]
fn test_find_git_repositories_setup_py() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("legacy-python");
    fs::create_dir_all(&repo_path).unwrap();

    // Create setup.py file (legacy Python project)
    fs::write(
        repo_path.join("setup.py"),
        "from setuptools import setup\nsetup()",
    )
    .unwrap();
    create_git_repo(
        &repo_path,
        Some("https://github.com/user/legacy-python.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"python".to_string()));
}

#[test]
fn test_find_git_repositories_build_gradle() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("gradle-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create build.gradle file
    fs::write(repo_path.join("build.gradle"), "plugins { id 'java' }").unwrap();
    create_git_repo(
        &repo_path,
        Some("https://github.com/user/gradle-project.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"java".to_string()));
}

#[test]
fn test_find_git_repositories_main_go() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("go-main-project");
    fs::create_dir_all(&repo_path).unwrap();

    // Create main.go file (alternative to go.mod)
    fs::write(repo_path.join("main.go"), "package main\n\nfunc main() {}").unwrap();
    create_git_repo(
        &repo_path,
        Some("https://github.com/user/go-main-project.git"),
    )
    .unwrap();

    let repos = find_git_repositories(temp_dir.path().to_str().unwrap()).unwrap();
    assert_eq!(repos.len(), 1);
    assert!(repos[0].tags.contains(&"go".to_string()));
}
