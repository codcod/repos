//! Command pattern implementation for CLI operations

pub mod base;
pub mod clone;
pub mod init;
pub mod pr;
pub mod remove;
pub mod run;

// Re-export the base types and all commands
pub use base::{Command, CommandContext};
pub use clone::CloneCommand;
pub use init::InitCommand;
pub use pr::PrCommand;
pub use remove::RemoveCommand;
pub use run::RunCommand;
