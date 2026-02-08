//! Central constants for the repos application

/// Default values for Git operations
pub mod git {
    /// Default fallback branch name when unable to determine repository default branch
    pub const FALLBACK_BRANCH: &str = "main";

    /// Default commit message when none is provided
    pub const DEFAULT_COMMIT_MSG: &str = "Automated changes";
}

/// Default values for GitHub operations
pub mod github {
    /// Default prefix for automated branch names
    pub const DEFAULT_BRANCH_PREFIX: &str = "automated-changes";

    /// Length of UUID suffix used in branch names
    pub const UUID_LENGTH: usize = 6;

    /// GitHub API base URL
    pub const API_BASE: &str = "https://api.github.com";

    /// Default User-Agent header for API requests
    pub const DEFAULT_USER_AGENT: &str = concat!("repos/", env!("CARGO_PKG_VERSION"));
}

/// Default values for configuration
pub mod config {
    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "repos.yaml";

    /// Default output directory
    pub const DEFAULT_LOGS_DIR: &str = "output";
}
