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
