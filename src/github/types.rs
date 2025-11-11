//! GitHub workflow types
//!
//! This module contains workflow-specific types for GitHub operations.
//! For low-level GitHub API types, see the `repos-github` crate.

/// Pull request options for creation workflow
#[derive(Debug, Clone)]
pub struct PrOptions {
    pub title: String,
    pub body: String,
    pub branch_name: Option<String>,
    pub base_branch: Option<String>,
    pub commit_msg: Option<String>,
    pub draft: bool,
    pub token: String,
    pub create_only: bool,
}

impl PrOptions {
    pub fn new(title: String, body: String, token: String) -> Self {
        Self {
            title,
            body,
            branch_name: None,
            base_branch: None,
            commit_msg: None,
            draft: false,
            token,
            create_only: false,
        }
    }

    pub fn with_branch_name(mut self, branch_name: String) -> Self {
        self.branch_name = Some(branch_name);
        self
    }

    pub fn with_base_branch(mut self, base_branch: String) -> Self {
        self.base_branch = Some(base_branch);
        self
    }

    pub fn with_commit_message(mut self, commit_msg: String) -> Self {
        self.commit_msg = Some(commit_msg);
        self
    }

    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }

    pub fn create_only(mut self) -> Self {
        self.create_only = true;
        self
    }
}
