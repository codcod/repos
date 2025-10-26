//! Repository discovery utilities for detecting and analyzing Git repositories

use crate::config::Repository;
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Find all Git repositories in a directory tree
pub fn find_git_repositories(start_path: &str) -> Result<Vec<Repository>> {
    let mut repositories = Vec::new();

    for entry in WalkDir::new(start_path)
        .min_depth(1)
        .max_depth(3) // Limit depth to avoid deep scanning
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Check if this directory contains a .git folder
        if path.is_dir()
            && path.join(".git").exists()
            && let Some(repo) = create_repository_from_path(path)?
        {
            repositories.push(repo);
        }
    }

    Ok(repositories)
}

/// Get remote URL from a Git repository
pub fn get_remote_url(repo_path: &Path) -> Result<Option<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(repo_path)
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(Some(url));
    }

    Ok(None)
}

/// Detect tags from repository path based on files and directory names
pub fn detect_tags_from_path(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();

    // Check for common patterns in directory names or files
    let path_str = path.to_string_lossy().to_lowercase();

    // Language detection based on files
    if path.join("go.mod").exists() || path.join("main.go").exists() {
        tags.push("go".to_string());
    }
    if path.join("package.json").exists() {
        tags.push("javascript".to_string());
        tags.push("node".to_string());
    }
    if path.join("requirements.txt").exists()
        || path.join("setup.py").exists()
        || path.join("pyproject.toml").exists()
    {
        tags.push("python".to_string());
    }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        tags.push("java".to_string());
    }
    if path.join("Cargo.toml").exists() {
        tags.push("rust".to_string());
    }

    // Type detection based on directory names
    if path_str.contains("frontend") || path_str.contains("ui") || path_str.contains("web") {
        tags.push("frontend".to_string());
    }
    if path_str.contains("backend") || path_str.contains("api") || path_str.contains("server") {
        tags.push("backend".to_string());
    }
    if path_str.contains("mobile") || path_str.contains("android") || path_str.contains("ios") {
        tags.push("mobile".to_string());
    }

    tags
}

/// Create a Repository instance from a filesystem path
pub fn create_repository_from_path(path: &Path) -> Result<Option<Repository>> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    if let Some(name) = name {
        // Try to get remote URL
        let url = get_remote_url(path)?;

        if let Some(url) = url {
            // Try to determine tags based on directory name or other heuristics
            let tags = detect_tags_from_path(path);

            let repository = Repository {
                name,
                url,
                tags,
                path: Some(path.to_string_lossy().to_string()),
                branch: None,
                config_dir: None, // Will be set when config is loaded
            };

            return Ok(Some(repository));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
    fn test_detect_tags_from_path_go() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("go-project");
        fs::create_dir_all(&repo_path).unwrap();
        fs::write(repo_path.join("go.mod"), "module test\n\ngo 1.19").unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"go".to_string()));
    }

    #[test]
    fn test_detect_tags_from_path_javascript() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("js-project");
        fs::create_dir_all(&repo_path).unwrap();
        fs::write(
            repo_path.join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )
        .unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"javascript".to_string()));
        assert!(tags.contains(&"node".to_string()));
    }

    #[test]
    fn test_detect_tags_from_path_python() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("python-project");
        fs::create_dir_all(&repo_path).unwrap();
        fs::write(repo_path.join("requirements.txt"), "requests==2.28.1\n").unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"python".to_string()));
    }

    #[test]
    fn test_detect_tags_from_path_frontend() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("frontend-app");
        fs::create_dir_all(&repo_path).unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_detect_tags_from_path_backend() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("backend-api");
        fs::create_dir_all(&repo_path).unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_detect_tags_from_path_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("fullstack-frontend");
        fs::create_dir_all(&repo_path).unwrap();

        // Create multiple language files
        fs::write(repo_path.join("package.json"), r#"{"name": "test"}"#).unwrap();
        fs::write(repo_path.join("requirements.txt"), "django").unwrap();

        let tags = detect_tags_from_path(&repo_path);
        assert!(tags.contains(&"javascript".to_string()));
        assert!(tags.contains(&"node".to_string()));
        assert!(tags.contains(&"python".to_string()));
        assert!(tags.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_get_remote_url_with_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).unwrap();

        create_git_repo(&repo_path, Some("https://github.com/user/test-repo.git")).unwrap();

        let url = get_remote_url(&repo_path).unwrap();
        assert_eq!(
            url,
            Some("https://github.com/user/test-repo.git".to_string())
        );
    }

    #[test]
    fn test_get_remote_url_without_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("local-repo");
        fs::create_dir_all(&repo_path).unwrap();

        create_git_repo(&repo_path, None).unwrap();

        let url = get_remote_url(&repo_path).unwrap();
        assert_eq!(url, None);
    }

    #[test]
    fn test_create_repository_from_path_with_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).unwrap();
        fs::write(repo_path.join("go.mod"), "module test").unwrap();

        create_git_repo(&repo_path, Some("https://github.com/user/test-repo.git")).unwrap();

        let repo = create_repository_from_path(&repo_path).unwrap();
        assert!(repo.is_some());
        let repo = repo.unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.url, "https://github.com/user/test-repo.git");
        assert!(repo.tags.contains(&"go".to_string()));
    }

    #[test]
    fn test_create_repository_from_path_without_remote() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("local-repo");
        fs::create_dir_all(&repo_path).unwrap();

        create_git_repo(&repo_path, None).unwrap();

        let repo = create_repository_from_path(&repo_path).unwrap();
        assert!(repo.is_none());
    }

    // Tests for find_git_repositories function
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
}
