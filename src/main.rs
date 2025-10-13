use anyhow::Result;
use clap::{Parser, Subcommand};
use repos::{commands::*, config::Config, constants};
use std::env;

#[derive(Parser)]
#[command(name = "repos")]
#[command(about = "A cli tool to manage multiple GitHub repositories")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone repositories specified in config
    Clone {
        /// Specific repository names to clone (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// Run a command in each repository
    Run {
        /// Command to execute
        command: String,

        /// Specific repository names to run command in (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Directory to store log files
        #[arg(short, long, default_value_t = constants::config::DEFAULT_LOGS_DIR.to_string())]
        logs: String,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// Create pull requests for repositories with changes
    Pr {
        /// Specific repository names to create PRs for (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Title for the pull request
        #[arg(long, default_value = "Automated changes")]
        title: String,

        /// Body text for the pull request
        #[arg(long, default_value = "This PR was created automatically")]
        body: String,

        /// Branch name to create
        #[arg(long)]
        branch: Option<String>,

        /// Base branch for the PR
        #[arg(long)]
        base: Option<String>,

        /// Commit message
        #[arg(long)]
        message: Option<String>,

        /// Create PR as draft
        #[arg(long)]
        draft: bool,

        /// GitHub token
        #[arg(long)]
        token: Option<String>,

        /// Only create PR, don't commit changes
        #[arg(long)]
        create_only: bool,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// Remove cloned repositories
    Rm {
        /// Specific repository names to remove (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// Create a config.yaml file from discovered Git repositories
    Init {
        /// Output file name
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        output: String,

        /// Overwrite existing file if it exists
        #[arg(long)]
        overwrite: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Execute the appropriate command
    match cli.command {
        Commands::Clone {
            repos,
            config,
            tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;
            let context = CommandContext {
                config,
                tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            CloneCommand.execute(&context).await?;
        }
        Commands::Run {
            command,
            repos,
            logs,
            config,
            tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;
            let context = CommandContext {
                config,
                tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            RunCommand {
                command,
                log_dir: logs,
            }
            .execute(&context)
            .await?;
        }
        Commands::Pr {
            repos,
            title,
            body,
            branch,
            base,
            message,
            draft,
            token,
            create_only,
            config,
            tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;
            let context = CommandContext {
                config,
                tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };

            let token = token.or_else(|| env::var("GITHUB_TOKEN").ok())
                .ok_or_else(|| anyhow::anyhow!("GitHub token not provided. Use --token flag or set GITHUB_TOKEN environment variable."))?;

            PrCommand {
                title,
                body,
                branch_name: branch,
                base_branch: base,
                commit_msg: message,
                draft,
                token,
                create_only,
            }
            .execute(&context)
            .await?;
        }
        Commands::Rm {
            repos,
            config,
            tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;
            let context = CommandContext {
                config,
                tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            RemoveCommand.execute(&context).await?;
        }
        Commands::Init { output, overwrite } => {
            // Init command doesn't need config since it creates one
            let context = CommandContext {
                config: Config::new(),
                tag: None,
                parallel: false,
                repos: None,
            };
            InitCommand { output, overwrite }.execute(&context).await?;
        }
    }

    Ok(())
}
