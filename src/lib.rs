//! Repos - A CLI tool for managing multiple GitHub repositories

pub mod commands;
pub mod config;
pub mod constants;
pub mod git;
pub mod github;
pub mod plugins;
pub mod runner;
pub mod util;

pub type Result<T> = anyhow::Result<T>;

// Re-export commonly used types
pub use commands::{Command, CommandContext};
pub use config::{Config, Repository};
pub use github::PrOptions;
