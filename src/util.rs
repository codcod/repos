//! Utility functions for repository discovery and file system operations

use crate::config::Repository;
use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

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
        if path.is_dir() && path.join(".git").exists() {
            if let Some(repo) = create_repository_from_path(path)? {
                repositories.push(repo);
            }
        }
    }

    Ok(repositories)
}

fn create_repository_from_path(path: &Path) -> Result<Option<Repository>> {
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

fn get_remote_url(repo_path: &Path) -> Result<Option<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(repo_path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(Some(url));
        }
    }

    Ok(None)
}

fn detect_tags_from_path(path: &Path) -> Vec<String> {
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

#[allow(dead_code)]
pub fn ensure_directory_exists(path: &str) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}
