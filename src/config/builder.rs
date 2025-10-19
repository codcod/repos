//! Repository builder utilities

use super::Repository;

/// Builder for creating repository configurations
pub struct RepositoryBuilder {
    name: String,
    url: String,
    tags: Vec<String>,
    path: Option<String>,
    branch: Option<String>,
}

impl RepositoryBuilder {
    /// Create a new repository builder
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            tags: Vec::new(),
            path: None,
            branch: None,
        }
    }

    /// Add tags to the repository
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the path for the repository
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Set the branch for the repository
    pub fn with_branch(mut self, branch: String) -> Self {
        self.branch = Some(branch);
        self
    }

    /// Build the repository
    pub fn build(self) -> Repository {
        Repository {
            name: self.name,
            url: self.url,
            tags: self.tags,
            path: self.path,
            branch: self.branch,
            config_dir: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_builder_basic_creation() {
        let builder = RepositoryBuilder::new(
            "test-repo".to_string(),
            "https://github.com/user/test-repo.git".to_string(),
        );
        let repo = builder.build();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.url, "https://github.com/user/test-repo.git");
        assert!(repo.tags.is_empty());
        assert!(repo.path.is_none());
    }

    #[test]
    fn test_repository_builder_with_tags() {
        let tags = vec!["backend".to_string(), "rust".to_string()];
        let repo = RepositoryBuilder::new(
            "backend-service".to_string(),
            "https://github.com/company/backend-service.git".to_string(),
        )
        .with_tags(tags.clone())
        .build();
        assert_eq!(repo.tags, tags);
    }

    #[test]
    fn test_repository_builder_with_path() {
        let repo = RepositoryBuilder::new(
            "local-repo".to_string(),
            "https://github.com/user/local-repo.git".to_string(),
        )
        .with_path("./local-path".to_string())
        .build();
        assert_eq!(repo.path, Some("./local-path".to_string()));
    }

    #[test]
    fn test_repository_builder_with_branch() {
        let repo = RepositoryBuilder::new(
            "feature-repo".to_string(),
            "https://github.com/user/feature-repo.git".to_string(),
        )
        .with_branch("feature-branch".to_string())
        .build();
        assert_eq!(repo.branch, Some("feature-branch".to_string()));
    }

    #[test]
    fn test_repository_builder_with_all_options() {
        let tags = vec!["frontend".to_string(), "javascript".to_string()];
        let repo = RepositoryBuilder::new(
            "full-repo".to_string(),
            "https://github.com/company/full-repo.git".to_string(),
        )
        .with_tags(tags.clone())
        .with_path("./frontend/full".to_string())
        .with_branch("develop".to_string())
        .build();
        assert_eq!(repo.tags, tags);
        assert_eq!(repo.path, Some("./frontend/full".to_string()));
        assert_eq!(repo.branch, Some("develop".to_string()));
    }

    #[test]
    fn test_repository_builder_overwrite_values() {
        let repo = RepositoryBuilder::new(
            "overwrite-test".to_string(),
            "https://github.com/user/overwrite-test.git".to_string(),
        )
        .with_path("./first-path".to_string())
        .with_path("./second-path".to_string())
        .with_branch("first-branch".to_string())
        .with_branch("second-branch".to_string())
        .with_tags(vec!["first-tag".to_string()])
        .with_tags(vec!["second-tag".to_string()])
        .build();
        assert_eq!(repo.path, Some("./second-path".to_string()));
        assert_eq!(repo.branch, Some("second-branch".to_string()));
        assert_eq!(repo.tags, vec!["second-tag".to_string()]);
    }
}
