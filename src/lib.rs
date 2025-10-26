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
pub use config::{Config, Repository};
pub use github::PrOptions;

/// Helper function for plugins to load the default config
pub fn load_default_config() -> anyhow::Result<Config> {
    Config::load_config(constants::config::DEFAULT_CONFIG_FILE)
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
