use crate::agent::CursorAgentRunner;
use crate::analysis::ProjectAnalyzer;
use crate::jira::{JiraClient, JiraTicket, parse_jira_input};
use crate::prompt::PromptGenerator;
use crate::workspace::{RepoManager, WorkspaceManager};
use anyhow::{Context, Result};
use colored::Colorize;
use repos::Repository;
use std::path::{Path, PathBuf};

pub struct FixWorkflow {
    repos: Vec<Repository>,
    ticket: String,
    ask_mode: bool,
    workspace_dir: Option<PathBuf>,
    additional_prompt: Option<String>,
    num_comments: usize,
    debug: bool,
}

impl FixWorkflow {
    pub fn new(
        repos: Vec<Repository>,
        ticket: String,
        ask_mode: bool,
        workspace_dir: Option<PathBuf>,
        additional_prompt: Option<String>,
        num_comments: usize,
        debug: bool,
    ) -> Self {
        Self {
            repos,
            ticket,
            ask_mode,
            workspace_dir,
            additional_prompt,
            num_comments,
            debug,
        }
    }

    pub fn run(&self, selected_repo_names: &[String]) -> Result<()> {
        let selected_repos = self.select_repositories(selected_repo_names)?;

        for repo in selected_repos {
            self.process_repository(repo)?;
        }

        Ok(())
    }

    fn select_repositories(&self, names: &[String]) -> Result<Vec<&Repository>> {
        if !names.is_empty() {
            // Filter to specified repos
            if self.debug {
                eprintln!("Filtering to specified repos: {:?}", names);
            }

            names
                .iter()
                .map(|repo_name| {
                    self.repos
                        .iter()
                        .find(|r| r.name == *repo_name)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Repository '{}' not found in filtered context. Available repos: {}",
                                repo_name,
                                self.repos
                                    .iter()
                                    .map(|r| r.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        })
                })
                .collect::<Result<Vec<_>>>()
        } else if self.repos.len() == 1 {
            // Single repo in context, use it
            Ok(vec![&self.repos[0]])
        } else if self.repos.is_empty() {
            anyhow::bail!(
                "No repositories in filtered context. Use tags (-t/--tag) to filter, or specify repository names as arguments."
            );
        } else {
            // Multiple repos in context, require explicit selection
            anyhow::bail!(
                "Multiple repositories match the filter ({}). Please specify which repository to fix:\n  repos fix <repo-name> --ticket {}\n\nAvailable repositories:\n{}",
                self.repos.len(),
                self.ticket,
                self.repos
                    .iter()
                    .map(|r| format!("  - {}", r.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }

    fn process_repository(&self, repo: &Repository) -> Result<()> {
        self.print_header();

        // Step 1: Fetch JIRA ticket
        let jira_ticket = self.fetch_jira_ticket()?;

        // Step 2: Setup workspace
        let workspace_manager = self.setup_workspace(&jira_ticket.key)?;
        let ticket_dir = workspace_manager.get_ticket_dir();

        // Step 3: Setup repository
        let repo_dir = self.setup_repository(repo, workspace_manager)?;

        // Step 4: Analyze project
        let analysis = self.analyze_project(&repo_dir)?;

        // Step 5: Generate prompts and context
        self.generate_artifacts(&jira_ticket, &analysis, &ticket_dir, repo, &repo_dir)?;

        // Step 6: Run cursor-agent
        let agent_runner = CursorAgentRunner::new()?;
        self.run_agent(&agent_runner, &ticket_dir, &jira_ticket, &analysis)?;

        // Verify and report
        self.verify_and_report(&agent_runner, &ticket_dir, &jira_ticket.key, &repo_dir)?;

        Ok(())
    }

    fn print_header(&self) {
        println!("{}", "=".repeat(60));
        println!("{}", "🤖 Repos Fix - Automated JIRA Ticket Resolver".bold());
        println!("{}", "=".repeat(60));
        println!();
    }

    fn fetch_jira_ticket(&self) -> Result<JiraTicket> {
        println!("{}", "Step 1: Fetching JIRA ticket...".bold().cyan());
        let (base_url, ticket_id) = parse_jira_input(&self.ticket)?;
        let jira_client = JiraClient::with_base_url(base_url)?;
        let ticket = jira_client.get_ticket(&ticket_id, self.num_comments)?;

        println!(
            "  {} Ticket: {} - {}",
            "✓".green(),
            ticket.key,
            ticket.title
        );
        println!("  {} Priority: {}", "✓".green(), ticket.priority);
        println!(
            "  {} Attachments: {}",
            "✓".green(),
            ticket.attachments.len()
        );
        println!();

        Ok(ticket)
    }

    fn setup_workspace(&self, ticket_id: &str) -> Result<WorkspaceManager> {
        println!("{}", "Step 2: Setting up workspace...".bold().cyan());
        let workspace_manager =
            WorkspaceManager::new(self.workspace_dir.as_deref(), ticket_id.to_string());
        workspace_manager.setup()?;
        let ticket_dir = workspace_manager.get_ticket_dir();
        println!("  {} Workspace: {}", "✓".green(), ticket_dir.display());
        println!();

        Ok(workspace_manager)
    }

    fn setup_repository(
        &self,
        repo: &Repository,
        _workspace_manager: WorkspaceManager,
    ) -> Result<PathBuf> {
        println!("{}", "Step 3: Setting up repository...".bold().cyan());
        let repo_manager = RepoManager::new(repo);
        let repo_dir = repo_manager.setup_repository()?;
        println!("  {} Repository: {}", "✓".green(), repo_dir.display());
        println!();

        Ok(repo_dir)
    }

    fn analyze_project(&self, repo_dir: &Path) -> Result<crate::analysis::ProjectAnalysis> {
        println!("{}", "Step 4: Analyzing project...".bold().cyan());
        let analyzer = ProjectAnalyzer::new(repo_dir);
        let analysis = analyzer.analyze()?;

        println!(
            "  {} Platform: {}",
            "✓".green(),
            analysis.platform.platform_type.as_str().to_uppercase()
        );
        println!(
            "  {} Languages: {}",
            "✓".green(),
            analysis
                .platform
                .languages
                .iter()
                .map(|l| l.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        if !analysis
            .architecture_patterns
            .dependency_injection
            .is_empty()
        {
            println!(
                "  {} DI Framework: {}",
                "✓".green(),
                analysis
                    .architecture_patterns
                    .dependency_injection
                    .join(", ")
            );
        }

        if !analysis.architecture_patterns.reactive.is_empty() {
            println!(
                "  {} Reactive: {}",
                "✓".green(),
                analysis.architecture_patterns.reactive.join(", ")
            );
        }

        if !analysis.test_structure.test_frameworks.is_empty() {
            println!(
                "  {} Test Framework: {}",
                "✓".green(),
                analysis
                    .test_structure
                    .test_frameworks
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!();

        Ok(analysis)
    }

    fn generate_artifacts(
        &self,
        ticket: &JiraTicket,
        analysis: &crate::analysis::ProjectAnalysis,
        ticket_dir: &Path,
        repo: &Repository,
        repo_dir: &Path,
    ) -> Result<()> {
        println!(
            "{}",
            "Step 5: Generating context and prompts...".bold().cyan()
        );

        // Save context
        let context = serde_json::json!({
            "ticket": ticket,
            "repository": {
                "name": repo.name,
                "url": repo.url,
                "path": repo_dir.to_string_lossy()
            },
            "analysis": analysis,
            "mode": if self.ask_mode { "ask" } else { "implementation" },
            "workspace": ticket_dir.to_string_lossy()
        });

        let context_str =
            serde_json::to_string_pretty(&context).context("Failed to serialize context")?;
        PromptGenerator::save_to_file(&context_str, ticket_dir, "mission-context.json")?;

        // Generate and save cursor prompt
        let cursor_prompt = PromptGenerator::generate_cursor_prompt(
            ticket,
            analysis,
            self.additional_prompt.as_deref(),
        )?;
        PromptGenerator::save_to_file(&cursor_prompt, ticket_dir, "cursor_prompt.md")?;

        // Generate and save cursorrules
        let cursorrules = PromptGenerator::generate_cursorrules(ticket, analysis, self.ask_mode)?;
        PromptGenerator::save_to_file(&cursorrules, ticket_dir, ".cursorrules")?;

        // Generate agent prompt
        let agent_prompt = PromptGenerator::generate_agent_prompt(
            ticket,
            analysis,
            self.ask_mode,
            self.additional_prompt.as_deref(),
        )?;
        PromptGenerator::save_to_file(&agent_prompt, ticket_dir, "agent_prompt.md")?;

        println!();
        Ok(())
    }

    fn run_agent(
        &self,
        agent_runner: &CursorAgentRunner,
        ticket_dir: &Path,
        ticket: &JiraTicket,
        analysis: &crate::analysis::ProjectAnalysis,
    ) -> Result<()> {
        println!("{}", "Step 6: Running cursor-agent...".bold().cyan());

        let agent_prompt = PromptGenerator::generate_agent_prompt(
            ticket,
            analysis,
            self.ask_mode,
            self.additional_prompt.as_deref(),
        )?;
        agent_runner.run_with_retry(ticket_dir, &agent_prompt, self.ask_mode, 3)?;

        Ok(())
    }

    fn verify_and_report(
        &self,
        agent_runner: &CursorAgentRunner,
        ticket_dir: &Path,
        ticket_id: &str,
        repo_dir: &Path,
    ) -> Result<()> {
        if agent_runner.verify_solution(ticket_dir)? {
            println!();
            println!("{}", "=".repeat(60));
            println!("{}", "✅ Task completed successfully!".bold().green());
            println!("{}", "=".repeat(60));
            println!();
            println!("📁 Workspace: {}", ticket_dir.display());
            println!("🌿 Branch: {}", ticket_id);
            println!("💻 Repository: {}", repo_dir.display());
            println!();
            println!("📋 Generated files:");
            println!("  • .cursorrules - Agent behavior rules");
            println!("  • mission-context.json - Complete analysis data");
            println!("  • cursor_prompt.md - Implementation guidelines, the 'rulebook' for Cursor");
            println!("  • agent_prompt.md - The 'mission' for Cursor Agent");
            println!("  • ANALYSIS.md - Pre-change analysis and plan");
            println!("  • SOLUTION_SUMMARY.md - Solution details");
            println!();
        } else {
            eprintln!("{}", "⚠️  Solution incomplete or not verified".yellow());
            eprintln!(
                "Check the workspace for partial results: {}",
                ticket_dir.display()
            );
        }

        Ok(())
    }
}
