use anyhow::Result;
use clap::{Parser, Subcommand};
use repos::commands::validators;
use repos::{commands::*, config::Config, constants, plugins};
use std::{env, path::PathBuf};

#[derive(Parser)]
#[command(name = "repos")]
#[command(about = "A cli tool to manage multiple GitHub repositories")]
#[command(version)]
#[command(allow_external_subcommands = true)]
struct Cli {
    /// List all available external plugins
    #[arg(long)]
    list_plugins: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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

        /// Filter repositories by tag (can be specified multiple times)
        #[arg(short, long)]
        tag: Vec<String>,

        /// Exclude repositories with these tags (can be specified multiple times)
        #[arg(short = 'e', long)]
        exclude_tag: Vec<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// Run a command in each repository
    Run {
        /// Command to execute
        #[arg(value_name = "COMMAND", help = "Command to execute")]
        command: Option<String>,

        /// Name of a recipe defined in config.yaml
        #[arg(long, help = "Name of a recipe defined in config.yaml")]
        recipe: Option<String>,

        /// Specific repository names to run command in (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag (can be specified multiple times)
        #[arg(short, long)]
        tag: Vec<String>,

        /// Exclude repositories with these tags (can be specified multiple times)
        #[arg(short = 'e', long)]
        exclude_tag: Vec<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,

        /// Don't save command outputs to files
        #[arg(long)]
        no_save: bool,

        /// Custom directory for output files (default: output)
        #[arg(long)]
        output_dir: Option<String>,
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

        /// Filter repositories by tag (can be specified multiple times)
        #[arg(short, long)]
        tag: Vec<String>,

        /// Exclude repositories with these tags (can be specified multiple times)
        #[arg(short = 'e', long)]
        exclude_tag: Vec<String>,

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

        /// Filter repositories by tag (can be specified multiple times)
        #[arg(short, long)]
        tag: Vec<String>,

        /// Exclude repositories with these tags (can be specified multiple times)
        #[arg(short = 'e', long)]
        exclude_tag: Vec<String>,

        /// Execute operations in parallel
        #[arg(short, long)]
        parallel: bool,
    },

    /// List repositories with optional filtering
    Ls {
        /// Specific repository names to list (if not provided, uses tag filter or all repos)
        repos: Vec<String>,

        /// Configuration file path
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        config: String,

        /// Filter repositories by tag (can be specified multiple times)
        #[arg(short, long)]
        tag: Vec<String>,

        /// Exclude repositories with these tags (can be specified multiple times)
        #[arg(short = 'e', long)]
        exclude_tag: Vec<String>,

        /// Output in JSON format for machine consumption
        #[arg(long)]
        json: bool,
    },

    /// Create a config.yaml file from discovered Git repositories
    Init {
        /// Output file name
        #[arg(short, long, default_value_t = constants::config::DEFAULT_CONFIG_FILE.to_string())]
        output: String,

        /// Overwrite existing file if it exists
        #[arg(long)]
        overwrite: bool,

        /// Supplement existing config with newly discovered repositories
        #[arg(long)]
        supplement: bool,
    },

    /// External plugin command
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle list-plugins option first
    if cli.list_plugins {
        let plugins = plugins::list_external_plugins();
        if plugins.is_empty() {
            println!("No external plugins found.");
            println!(
                "To create a plugin, make an executable named 'repos-<name>' available in your PATH."
            );
        } else {
            println!("Available external plugins:");
            for plugin in plugins {
                println!("  {}", plugin);
            }
        }
        return Ok(());
    }

    // Handle commands
    match cli.command {
        Some(Commands::External(args)) => {
            if args.is_empty() {
                anyhow::bail!("External command provided but no arguments given");
            }

            let plugin_name = &args[0];

            // Parse common options from plugin args
            let mut config_path = constants::config::DEFAULT_CONFIG_FILE.to_string();
            let mut include_tags = Vec::new();
            let mut exclude_tags = Vec::new();
            let mut debug = false;
            let mut plugin_args = Vec::new();

            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--config" | "-c" => {
                        if i + 1 < args.len() {
                            config_path = args[i + 1].clone();
                            i += 2;
                        } else {
                            anyhow::bail!("--config requires a path argument");
                        }
                    }
                    "--tag" | "-t" => {
                        if i + 1 < args.len() {
                            include_tags.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            anyhow::bail!("--tag requires a tag argument");
                        }
                    }
                    "--exclude-tag" | "-e" => {
                        if i + 1 < args.len() {
                            exclude_tags.push(args[i + 1].clone());
                            i += 2;
                        } else {
                            anyhow::bail!("--exclude-tag requires a tag argument");
                        }
                    }
                    "--debug" | "-d" => {
                        debug = true;
                        i += 1;
                    }
                    _ => {
                        // Plugin-specific arg
                        plugin_args.push(args[i].clone());
                        i += 1;
                    }
                }
            }

            // Load config and filter repositories (only if needed or if config exists)
            let needs_config = !include_tags.is_empty()
                || !exclude_tags.is_empty()
                || std::path::Path::new(&config_path).exists();

            let (config, filtered_repos) = if needs_config {
                let config = Config::load_config(&config_path)?;
                let filtered_repos = if include_tags.is_empty() && exclude_tags.is_empty() {
                    config.repositories.clone()
                } else {
                    config.filter_repositories(&include_tags, &exclude_tags, None)
                };
                (config, filtered_repos)
            } else {
                // No config available, pass empty data
                (Config::new(), Vec::new())
            };

            // Build plugin context
            let context = plugins::PluginContext::new(config, filtered_repos, plugin_args, debug);

            plugins::try_external_plugin(plugin_name, &context)?;
        }
        Some(command) => execute_builtin_command(command).await?,
        None => {
            // No command provided, print help
            anyhow::bail!("No command provided. Use --help for usage information.");
        }
    }

    Ok(())
}

async fn execute_builtin_command(command: Commands) -> Result<()> {
    // Execute the appropriate command
    match command {
        Commands::External(_) => {
            // These cases are handled in main(), this should not be reached
            unreachable!("External commands should be handled in main()")
        }
        Commands::Clone {
            repos,
            config,
            tag,
            exclude_tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;

            // Validate clone command arguments using centralized validators
            validators::validate_tag_filters(&tag)?;
            validators::validate_tag_filters(&exclude_tag)?;
            validators::validate_repository_names(&repos)?;

            let context = CommandContext {
                config,
                tag,
                exclude_tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            CloneCommand.execute(&context).await?;
        }
        Commands::Run {
            command,
            recipe,
            repos,
            config,
            tag,
            exclude_tag,
            parallel,
            no_save,
            output_dir,
        } => {
            let config = Config::load_config(&config)?;

            // Validate run command arguments using centralized validators
            validators::validate_run_args(&command, &recipe)?;
            validators::validate_tag_filters(&tag)?;
            validators::validate_tag_filters(&exclude_tag)?;
            validators::validate_repository_names(&repos)?;
            validators::validate_output_directory(&output_dir)?;

            let context = CommandContext {
                config,
                tag,
                exclude_tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };

            if let Some(cmd) = command {
                RunCommand::new_command(cmd, no_save, output_dir.map(PathBuf::from))
                    .execute(&context)
                    .await?;
            } else if let Some(recipe_name) = recipe {
                RunCommand::new_recipe(recipe_name, no_save, output_dir.map(PathBuf::from))
                    .execute(&context)
                    .await?;
            }
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
            exclude_tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;

            // Validate PR command arguments using centralized validators
            validators::validate_pr_args(&token)?;
            validators::validate_tag_filters(&tag)?;
            validators::validate_tag_filters(&exclude_tag)?;
            validators::validate_repository_names(&repos)?;
            validators::validate_branch_name(&branch)?;
            validators::validate_branch_name(&base)?;
            validators::validate_commit_message(&message)?;

            let context = CommandContext {
                config,
                tag,
                exclude_tag,
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
            exclude_tag,
            parallel,
        } => {
            let config = Config::load_config(&config)?;

            // Validate remove command arguments using centralized validators
            validators::validate_tag_filters(&tag)?;
            validators::validate_tag_filters(&exclude_tag)?;
            validators::validate_repository_names(&repos)?;

            let context = CommandContext {
                config,
                tag,
                exclude_tag,
                parallel,
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            RemoveCommand.execute(&context).await?;
        }
        Commands::Ls {
            repos,
            config,
            tag,
            exclude_tag,
            json,
        } => {
            let config = Config::load_config(&config)?;

            // Validate list command arguments using centralized validators
            validators::validate_tag_filters(&tag)?;
            validators::validate_tag_filters(&exclude_tag)?;
            validators::validate_repository_names(&repos)?;

            let context = CommandContext {
                config,
                tag,
                exclude_tag,
                parallel: false, // List command doesn't need parallel execution
                repos: if repos.is_empty() { None } else { Some(repos) },
            };
            ListCommand { json }.execute(&context).await?;
        }
        Commands::Init {
            output,
            overwrite,
            supplement,
        } => {
            // Init command doesn't need config since it creates one
            let context = CommandContext {
                config: Config::new(),
                tag: Vec::new(),
                exclude_tag: Vec::new(),
                parallel: false,
                repos: None,
            };
            InitCommand {
                output,
                overwrite,
                supplement,
            }
            .execute(&context)
            .await?;
        }
    }

    Ok(())
}
