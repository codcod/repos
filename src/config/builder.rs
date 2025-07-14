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
