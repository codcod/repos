use crate::agent::CursorAgentRunner;
use crate::analysis::ProjectAnalyzer;
use crate::jira::{JiraClient, JiraTicket, parse_jira_input};
use crate::prompt::{KnowledgeContext, PromptGenerator};
use crate::workspace::{RepoManager, WorkspaceManager};
use anyhow::{Context, Result};
use colored::Colorize;
use repos::Repository;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FixWorkflow {
    repos: Vec<Repository>,
    ticket: String,
    ask_mode: bool,
    workspace_dir: Option<PathBuf>,
    additional_prompt: Option<String>,
    knowledge_dir: Option<PathBuf>,
    num_comments: usize,
    debug: bool,
}

impl FixWorkflow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repos: Vec<Repository>,
        ticket: String,
        ask_mode: bool,
        workspace_dir: Option<PathBuf>,
        additional_prompt: Option<String>,
        knowledge_dir: Option<PathBuf>,
        num_comments: usize,
        debug: bool,
    ) -> Self {
        Self {
            repos,
            ticket,
            ask_mode,
            workspace_dir,
            additional_prompt,
            knowledge_dir,
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

        // Step 5: Prepare knowledge base (optional)
        let knowledge = self.prepare_knowledge_base(&jira_ticket, &ticket_dir)?;

        // Step 6: Generate prompts and context
        self.generate_artifacts(
            &jira_ticket,
            &analysis,
            &ticket_dir,
            repo,
            &repo_dir,
            knowledge.as_ref(),
        )?;

        // Step 7: Run cursor-agent
        let agent_runner = CursorAgentRunner::new()?;
        self.run_agent(
            &agent_runner,
            &ticket_dir,
            &jira_ticket,
            &analysis,
            knowledge.as_ref(),
        )?;

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
        knowledge: Option<&KnowledgeContext>,
    ) -> Result<()> {
        println!(
            "{}",
            "Step 6: Generating context and prompts...".bold().cyan()
        );

        // Save context
        let knowledge_context = knowledge.map(|ctx| {
            serde_json::json!({
                "dir": ctx.dir_name,
                "files": ctx.files,
                "inline_files": ctx.inline_files
            })
        });
        let context = serde_json::json!({
            "ticket": ticket,
            "repository": {
                "name": repo.name,
                "url": repo.url,
                "path": repo_dir.to_string_lossy()
            },
            "analysis": analysis,
            "mode": if self.ask_mode { "ask" } else { "implementation" },
            "workspace": ticket_dir.to_string_lossy(),
            "knowledge_base": knowledge_context
        });

        let context_str =
            serde_json::to_string_pretty(&context).context("Failed to serialize context")?;
        PromptGenerator::save_to_file(&context_str, ticket_dir, "mission-context.json")?;

        // Generate and save cursor prompt
        let cursor_prompt = PromptGenerator::generate_cursor_prompt(
            ticket,
            analysis,
            self.additional_prompt.as_deref(),
            knowledge,
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
            knowledge,
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
        knowledge: Option<&KnowledgeContext>,
    ) -> Result<()> {
        println!("{}", "Step 7: Running cursor-agent...".bold().cyan());

        let agent_prompt = PromptGenerator::generate_agent_prompt(
            ticket,
            analysis,
            self.ask_mode,
            self.additional_prompt.as_deref(),
            knowledge,
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

    fn prepare_knowledge_base(
        &self,
        ticket: &JiraTicket,
        ticket_dir: &Path,
    ) -> Result<Option<KnowledgeContext>> {
        println!("{}", "Step 5: Preparing knowledge base...".bold().cyan());
        let knowledge_dir = match &self.knowledge_dir {
            Some(dir) => dir,
            None => {
                println!("  ℹ️  No knowledge base directory provided");
                println!();
                return Ok(None);
            }
        };

        let markdown_files = Self::list_markdown_files(knowledge_dir)?;
        if markdown_files.is_empty() {
            println!("  ⚠️  Knowledge base directory has no .md files");
            println!();
            return Ok(None);
        }

        let dest_dir = ticket_dir.join("knowledge");
        fs::create_dir_all(&dest_dir)
            .with_context(|| format!("Failed to create {}", dest_dir.display()))?;

        let mut file_contents = Vec::new();
        let mut copied_files = Vec::new();
        for path in markdown_files {
            let filename = path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("unknown.md")
                .to_string();
            fs::copy(&path, dest_dir.join(&filename))
                .with_context(|| format!("Failed to copy knowledge file {}", path.display()))?;
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read knowledge file {}", path.display()))?;
            copied_files.push(filename.clone());
            file_contents.push((filename, content));
        }

        copied_files.sort();
        file_contents.sort_by(|a, b| a.0.cmp(&b.0));

        let selection = Self::select_inline_knowledge(ticket, &file_contents);
        let inline_content = Self::build_inline_knowledge(&selection);

        println!("  {} Knowledge files: {}", "✓".green(), copied_files.len());
        if let Some(content) = &inline_content {
            println!(
                "  {} Inlined knowledge size: {} chars",
                "✓".green(),
                content.len()
            );
        }
        println!();

        Ok(Some(KnowledgeContext {
            dir_name: "knowledge".to_string(),
            files: copied_files,
            inline_files: selection.iter().map(|(name, _)| name.clone()).collect(),
            inline_content,
        }))
    }

    fn list_markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read knowledge directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|ext| ext.eq_ignore_ascii_case("md"))
                    .unwrap_or(false)
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    fn select_inline_knowledge(
        ticket: &JiraTicket,
        files: &[(String, String)],
    ) -> Vec<(String, String)> {
        const MAX_INLINE_FILES: usize = 4;
        const MAX_KEYWORDS: usize = 50;

        let mut keywords = Self::extract_keywords(ticket, MAX_KEYWORDS);
        if !ticket.key.is_empty() {
            keywords.push(ticket.key.to_lowercase());
        }
        let keyword_set: HashSet<String> = keywords.into_iter().collect();

        let mut scored: Vec<(usize, String, String)> = files
            .iter()
            .map(|(name, content)| {
                let mut score = 0usize;
                let name_lower = name.to_lowercase();
                let content_lower = content.to_lowercase();
                for keyword in &keyword_set {
                    if name_lower.contains(keyword) {
                        score += 2;
                    }
                    if content_lower.contains(keyword) {
                        score += 1;
                    }
                }
                (score, name.clone(), content.clone())
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

        let mut selected = Vec::new();
        for (score, name, content) in scored.into_iter() {
            if score == 0 && !selected.is_empty() {
                break;
            }
            selected.push((name, content));
            if selected.len() >= MAX_INLINE_FILES {
                break;
            }
        }

        if selected.is_empty() && !files.is_empty() {
            let (name, content) = &files[0];
            selected.push((name.clone(), content.clone()));
        }

        selected
    }

    fn build_inline_knowledge(files: &[(String, String)]) -> Option<String> {
        const MAX_INLINE_CHARS: usize = 12_000;
        const MAX_FILE_CHARS: usize = 4_000;

        if files.is_empty() {
            return None;
        }

        let mut combined = String::new();
        for (name, content) in files {
            if combined.len() >= MAX_INLINE_CHARS {
                break;
            }
            let mut snippet = content.trim().to_string();
            if snippet.len() > MAX_FILE_CHARS {
                snippet.truncate(MAX_FILE_CHARS);
                snippet.push_str("\n\n[Truncated]\n");
            }
            let entry = format!("## Knowledge Base: {}\n\n{}\n\n", name, snippet);
            if combined.len() + entry.len() > MAX_INLINE_CHARS {
                break;
            }
            combined.push_str(&entry);
        }

        if combined.trim().is_empty() {
            None
        } else {
            Some(combined)
        }
    }

    fn extract_keywords(ticket: &JiraTicket, max_keywords: usize) -> Vec<String> {
        let mut text = String::new();
        text.push_str(&ticket.title);
        text.push(' ');
        text.push_str(&ticket.description);
        text.push(' ');
        text.push_str(&ticket.issue_type);
        for label in &ticket.labels {
            text.push(' ');
            text.push_str(label);
        }

        let mut keywords = Vec::new();
        let stopwords = Self::stopwords();
        let mut seen = HashSet::new();
        for token in text
            .to_lowercase()
            .split(|ch: char| !ch.is_ascii_alphanumeric())
        {
            if token.len() < 4 || stopwords.contains(token) {
                continue;
            }
            if seen.insert(token.to_string()) {
                keywords.push(token.to_string());
                if keywords.len() >= max_keywords {
                    break;
                }
            }
        }
        keywords
    }

    fn stopwords() -> HashSet<&'static str> {
        HashMap::from([
            ("that", ""),
            ("this", ""),
            ("with", ""),
            ("from", ""),
            ("into", ""),
            ("your", ""),
            ("have", ""),
            ("will", ""),
            ("should", ""),
            ("could", ""),
            ("would", ""),
            ("there", ""),
            ("their", ""),
            ("about", ""),
            ("these", ""),
            ("those", ""),
            ("which", ""),
            ("while", ""),
            ("where", ""),
            ("what", ""),
            ("when", ""),
            ("like", ""),
            ("also", ""),
            ("only", ""),
            ("make", ""),
            ("just", ""),
        ])
        .keys()
        .copied()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_ticket(title: &str, description: &str, labels: Vec<&str>) -> JiraTicket {
        JiraTicket {
            id: "1".to_string(),
            key: "MAINT-1".to_string(),
            title: title.to_string(),
            description: description.to_string(),
            labels: labels.into_iter().map(|label| label.to_string()).collect(),
            status: "Open".to_string(),
            priority: "P2".to_string(),
            issue_type: "Bug".to_string(),
            assignee: "Unassigned".to_string(),
            reporter: "Reporter".to_string(),
            created: "2024-01-01".to_string(),
            updated: "2024-01-02".to_string(),
            attachments: Vec::new(),
            comments: Vec::new(),
        }
    }

    #[test]
    fn extract_keywords_filters_stopwords_and_short_tokens() {
        let ticket = make_ticket(
            "Fix payment timeout in checkout",
            "Timeout occurs when user tries to pay",
            vec!["payments", "urgent"],
        );
        let keywords = FixWorkflow::extract_keywords(&ticket, 50);

        assert!(keywords.contains(&"payment".to_string()) || keywords.contains(&"payments".to_string()));
        assert!(keywords.contains(&"timeout".to_string()));
        assert!(!keywords.contains(&"when".to_string()));

        let unique: HashSet<_> = keywords.iter().cloned().collect();
        assert_eq!(unique.len(), keywords.len());
    }

    #[test]
    fn select_inline_knowledge_scores_by_name_and_content() {
        let ticket = make_ticket(
            "Payment failure during checkout",
            "Timeout when processing payment",
            vec!["payments"],
        );
        let files = vec![
            ("payments-guide.md".to_string(), "Payment retries and timeouts".to_string()),
            ("checkout.md".to_string(), "Checkout flow details".to_string()),
            ("misc.md".to_string(), "Unrelated content".to_string()),
        ];

        let selected = FixWorkflow::select_inline_knowledge(&ticket, &files);
        assert!(!selected.is_empty());
        assert_eq!(selected[0].0, "payments-guide.md");
    }

    #[test]
    fn select_inline_knowledge_falls_back_to_first_file() {
        let ticket = make_ticket("No matching keywords", "Nothing in common", vec![]);
        let files = vec![
            ("alpha.md".to_string(), "first file".to_string()),
            ("beta.md".to_string(), "second file".to_string()),
        ];

        let selected = FixWorkflow::select_inline_knowledge(&ticket, &files);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0, "alpha.md");
    }

    #[test]
    fn build_inline_knowledge_truncates_long_entries() {
        let long_content = "a".repeat(5000);
        let files = vec![("guide.md".to_string(), long_content)];

        let inline = FixWorkflow::build_inline_knowledge(&files).expect("inline");
        assert!(inline.contains("## Knowledge Base: guide.md"));
        assert!(inline.contains("[Truncated]"));
    }

    #[test]
    fn build_inline_knowledge_empty_returns_none() {
        assert!(FixWorkflow::build_inline_knowledge(&[]).is_none());
    }

    #[test]
    fn list_markdown_files_only_returns_md() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let md_path = temp_dir.path().join("one.md");
        let txt_path = temp_dir.path().join("two.txt");
        fs::write(&md_path, "# doc").expect("write md");
        fs::write(&txt_path, "ignore").expect("write txt");

        let files = FixWorkflow::list_markdown_files(temp_dir.path()).unwrap();
        assert_eq!(files, vec![md_path]);
    }
}
