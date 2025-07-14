//! Base types and traits for the command pattern

use crate::config::Config;
use anyhow::Result;

/// Context passed to all commands containing shared configuration and options
#[derive(Clone)]
pub struct CommandContext {
    /// The loaded configuration
    pub config: Config,
    /// Optional tag filter for repositories
    pub tag: Option<String>,
    /// Whether to execute operations in parallel
    pub parallel: bool,
    /// Optional list of specific repository names to operate on
    pub repos: Option<Vec<String>>,
}

/// Trait that all commands must implement
#[async_trait::async_trait]
pub trait Command {
    /// Execute the command with the given context
    async fn execute(&self, context: &CommandContext) -> Result<()>;
}
