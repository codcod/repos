mod agent;
mod analysis;
mod domain;
mod jira;
mod prompt;
mod workflow;
mod workspace;

use anyhow::{Context, Result};
use clap::Parser;
use repos::{is_debug_mode, load_plugin_context};
use std::path::PathBuf;
use workflow::FixWorkflow;

#[derive(Parser, Debug)]
#[command(name = "repos-fix")]
#[command(about = "Automatically fix JIRA maintenance tickets using Cursor AI")]
struct Args {
    /// Repository names to fix (if not provided, uses filtered repos from context)
    repos: Vec<String>,

    /// JIRA ticket ID or full URL (e.g., MAINT-1234 or https://company.atlassian.net/browse/MAINT-1234)
    #[arg(long)]
    ticket: String,

    /// Ask mode - analyze only, no code changes
    #[arg(long)]
    ask: bool,

    /// Custom workspace directory
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Additional prompt to append to the generated prompt
    #[arg(short, long)]
    prompt: Option<String>,

    /// Number of recent JIRA comments to include in prompts
    #[arg(long, default_value_t = 10)]
    num_comments: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let debug = is_debug_mode();

    if debug {
        eprintln!("Debug mode enabled");
    }

    // Load repositories from injected context
    let repos = load_plugin_context()?
        .context("Failed to load plugin context. Make sure to run via 'repos fix'")?;

    if debug {
        eprintln!("Loaded {} repositories from context", repos.len());
    }

    // Create and run workflow
    let workflow = FixWorkflow::new(
        repos,
        args.ticket,
        args.ask,
        args.workspace,
        args.prompt,
        args.num_comments,
        debug,
    );

    workflow.run(&args.repos)?;

    Ok(())
}
