//! Repos - A CLI tool for managing multiple GitHub repositories

pub mod commands;
pub mod config;
pub mod constants;
pub mod git;
pub mod github;
pub mod plugins;
pub mod runner;
pub mod utils;

pub type Result<T> = anyhow::Result<T>;

// Re-export commonly used types
pub use commands::{Command, CommandContext};
pub use config::loader::save_config;
pub use config::{Config, Repository};
pub use github::PrOptions;
pub use plugins::PluginContext;

/// Helper function for plugins to load the default config
pub fn load_default_config() -> anyhow::Result<Config> {
    Config::load_config(constants::config::DEFAULT_CONFIG_FILE)
}

/// Helper function for plugins to load context from environment variables
///
/// External plugins executed by the core repos CLI will have access to:
/// - REPOS_PLUGIN_PROTOCOL: Set to "1" if context injection is enabled
/// - REPOS_FILTERED_REPOS_FILE: Path to JSON file with filtered repositories
/// - REPOS_DEBUG: Set to "1" if debug mode is enabled
/// - REPOS_TOTAL_REPOS: Total number of repositories in config
/// - REPOS_FILTERED_COUNT: Number of repositories after filtering
pub fn load_plugin_context() -> anyhow::Result<Option<Vec<Repository>>> {
    // Check if plugin protocol is enabled
    if std::env::var("REPOS_PLUGIN_PROTOCOL").ok().as_deref() != Some("1") {
        return Ok(None);
    }

    // Read filtered repositories from file
    let repos_file = std::env::var("REPOS_FILTERED_REPOS_FILE")
        .map_err(|_| anyhow::anyhow!("REPOS_FILTERED_REPOS_FILE not set"))?;

    let file_content = std::fs::read_to_string(&repos_file)
        .map_err(|e| anyhow::anyhow!("Failed to read repos file: {}", e))?;

    let repos: Vec<Repository> = serde_json::from_str(&file_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse repos JSON: {}", e))?;

    Ok(Some(repos))
}

/// Check if debug mode is enabled via environment variable
pub fn is_debug_mode() -> bool {
    std::env::var("REPOS_DEBUG").ok().as_deref() == Some("1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config_execution() {
        // Test that the function exists and can be called
        // This will likely fail since the default config file doesn't exist,
        // but it exercises the code path
        let result = load_default_config();

        // We expect this to fail since there's no default config file in test environment
        assert!(result.is_err());
    }

    #[test]
    fn test_lib_module_exists() {
        // Test that library module exports are accessible
        use crate::{CommandContext, PrOptions, Repository};

        // Just verify the types can be referenced
        let _: Option<CommandContext> = None;
        let _: Option<Repository> = None;
        let _: Option<PrOptions> = None;
    }
}
