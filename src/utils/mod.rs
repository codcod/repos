//! Utility modules for common functionality

pub mod exit_codes;
pub mod filesystem;
pub mod filters;
pub mod repository_discovery;
pub mod sanitizers;

// Re-export commonly used functions
pub use exit_codes::get_exit_code_description;
pub use filesystem::ensure_directory_exists;
pub use filters::{filter_by_names, filter_by_tag, filter_repositories};
pub use repository_discovery::{
    create_repository_from_path, detect_tags_from_path, find_git_repositories, get_remote_url,
};
pub use sanitizers::{sanitize_for_filename, sanitize_script_name};
