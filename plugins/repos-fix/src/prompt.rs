use crate::analysis::ProjectAnalysis;
use crate::domain::PlatformType;
use crate::jira::JiraTicket;
use anyhow::{Context, Result};
use minijinja::{Environment, context};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static TEMPLATE_ENV: OnceLock<Environment<'static>> = OnceLock::new();

#[derive(Clone, Debug, Default)]
pub struct KnowledgeContext {
    pub dir_name: String,
    pub files: Vec<String>,
    pub inline_files: Vec<String>,
    pub inline_content: Option<String>,
}

fn template_override_dir() -> Option<PathBuf> {
    let xdg_config = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let base = xdg_config.or_else(|| {
        env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(|home| PathBuf::from(home).join(".config"))
    });

    base.map(|base| base.join("repos").join("fix"))
}

fn read_override_template(filename: &str) -> Option<String> {
    let path = template_override_dir()?.join(filename);
    if path.is_file() {
        fs::read_to_string(&path).ok()
    } else {
        None
    }
}

fn load_template_source(filename: &str, fallback: &'static str) -> &'static str {
    if let Some(source) = read_override_template(filename) {
        Box::leak(source.into_boxed_str())
    } else {
        fallback
    }
}

fn load_guidelines(filename: &str, fallback: &'static str) -> String {
    read_override_template(filename).unwrap_or_else(|| fallback.to_string())
}

fn get_template_env() -> &'static Environment<'static> {
    TEMPLATE_ENV.get_or_init(|| {
        let mut env = Environment::new();

        // Load templates from embedded strings
        env.add_template(
            "cursor_prompt",
            load_template_source(
                "cursor_prompt.md",
                include_str!("templates/cursor_prompt.md"),
            ),
        )
        .expect("Failed to add cursor_prompt template");
        env.add_template(
            "cursorrules",
            load_template_source("cursorrules.md", include_str!("templates/cursorrules.md")),
        )
        .expect("Failed to add cursorrules template");
        env.add_template(
            "agent_prompt",
            load_template_source("agent_prompt.md", include_str!("templates/agent_prompt.md")),
        )
        .expect("Failed to add agent_prompt template");

        env
    })
}

pub struct PromptGenerator;

impl PromptGenerator {
    pub fn generate_cursor_prompt(
        ticket: &JiraTicket,
        analysis: &ProjectAnalysis,
        additional_prompt: Option<&str>,
        knowledge: Option<&KnowledgeContext>,
    ) -> Result<String> {
        let env = get_template_env();
        let tmpl = env.get_template("cursor_prompt")?;

        let platform_guidelines = Self::get_platform_guidelines(&analysis.platform.platform_type);
        let is_security_task = Self::is_security_task(ticket);
        let has_knowledge_base = knowledge.map(|ctx| !ctx.files.is_empty()).unwrap_or(false);

        let ctx = context! {
            platform_emoji => analysis.platform.platform_type.emoji(),
            ticket => ticket,
            platform_name => analysis.platform.platform_type.as_str().to_uppercase(),
            languages => analysis.platform.languages.iter()
                .map(|l| l.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            frameworks => analysis.platform.frameworks.iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            source_dirs => analysis.project_structure.source_directories
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            config_files => analysis.project_structure.config_files
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            has_di => !analysis.architecture_patterns.dependency_injection.is_empty(),
            di_frameworks => analysis.architecture_patterns.dependency_injection.join(", "),
            has_reactive => !analysis.architecture_patterns.reactive.is_empty(),
            reactive_frameworks => analysis.architecture_patterns.reactive.join(", "),
            has_ui => !analysis.architecture_patterns.ui_framework.is_empty(),
            ui_frameworks => analysis.architecture_patterns.ui_framework.join(", "),
            has_test_frameworks => !analysis.test_structure.test_frameworks.is_empty(),
            test_frameworks => analysis.test_structure.test_frameworks.iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            has_test_dirs => !analysis.test_structure.test_directories.is_empty(),
            test_dirs => analysis.test_structure.test_directories
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            platform_guidelines => platform_guidelines,
            main_build => analysis.build_commands.main_build,
            test_compile => analysis.build_commands.test_compile,
            test_run => analysis.build_commands.test_run,
            is_security_task => is_security_task,
            additional_prompt => additional_prompt,
            has_knowledge_base => has_knowledge_base,
            knowledge_base_dir => knowledge.map(|ctx| ctx.dir_name.as_str()).unwrap_or(""),
            knowledge_base_files => knowledge.map(|ctx| ctx.files.clone()).unwrap_or_default(),
            knowledge_base_inline_files => knowledge.map(|ctx| ctx.inline_files.clone()).unwrap_or_default(),
            knowledge_base_content => knowledge.and_then(|ctx| ctx.inline_content.as_deref()),
        };

        Ok(tmpl.render(ctx)?)
    }

    pub fn generate_cursorrules(
        ticket: &JiraTicket,
        analysis: &ProjectAnalysis,
        ask_mode: bool,
    ) -> Result<String> {
        let env = get_template_env();
        let tmpl = env.get_template("cursorrules")?;

        let test_step_num = if analysis.build_commands.test_compile.is_some() {
            "3"
        } else {
            "2"
        };
        let is_security_task = Self::is_security_task(ticket);

        let ctx = context! {
            mode_title => if ask_mode { "ASK Mode Analysis" } else { "Automated Maintenance Assistant" },
            ask_mode => ask_mode,
            ticket => ticket,
            platform_name => analysis.platform.platform_type.as_str().to_uppercase(),
            main_build => analysis.build_commands.main_build,
            test_compile => analysis.build_commands.test_compile,
            test_run => analysis.build_commands.test_run,
            test_step_num => test_step_num,
            is_security_task => is_security_task,
        };

        Ok(tmpl.render(ctx)?)
    }

    pub fn generate_agent_prompt(
        ticket: &JiraTicket,
        analysis: &ProjectAnalysis,
        ask_mode: bool,
        additional_prompt: Option<&str>,
        knowledge: Option<&KnowledgeContext>,
    ) -> Result<String> {
        let env = get_template_env();
        let tmpl = env.get_template("agent_prompt")?;

        let test_run_step = if analysis.build_commands.test_compile.is_some() {
            "8"
        } else {
            "7"
        };
        let is_security_task = Self::is_security_task(ticket);
        let has_knowledge_base = knowledge.map(|ctx| !ctx.files.is_empty()).unwrap_or(false);

        let ctx = context! {
            ask_mode => ask_mode,
            ticket => ticket,
            main_build => analysis.build_commands.main_build,
            test_compile => analysis.build_commands.test_compile,
            test_run => analysis.build_commands.test_run,
            test_run_step => test_run_step,
            is_security_task => is_security_task,
            additional_prompt => additional_prompt,
            has_knowledge_base => has_knowledge_base,
            knowledge_base_dir => knowledge.map(|ctx| ctx.dir_name.as_str()).unwrap_or(""),
            knowledge_base_files => knowledge.map(|ctx| ctx.files.clone()).unwrap_or_default(),
            knowledge_base_inline_files => knowledge.map(|ctx| ctx.inline_files.clone()).unwrap_or_default(),
        };

        Ok(tmpl.render(ctx)?)
    }

    pub fn save_to_file(content: &str, path: &Path, filename: &str) -> Result<()> {
        let file_path = path.join(filename);
        fs::write(&file_path, content).with_context(|| format!("Failed to write {}", filename))?;
        println!("Created: {}", file_path.display());
        Ok(())
    }

    fn get_platform_guidelines(platform: &PlatformType) -> String {
        match platform {
            PlatformType::Ios => load_guidelines(
                "guidelines_ios.md",
                include_str!("templates/guidelines_ios.md"),
            ),
            PlatformType::Android => load_guidelines(
                "guidelines_android.md",
                include_str!("templates/guidelines_android.md"),
            ),
            PlatformType::Java => load_guidelines(
                "guidelines_java.md",
                include_str!("templates/guidelines_java.md"),
            ),
            PlatformType::Angular => load_guidelines(
                "guidelines_angular.md",
                include_str!("templates/guidelines_angular.md"),
            ),
            PlatformType::Unknown => String::new(),
        }
    }

    fn is_security_task(ticket: &JiraTicket) -> bool {
        let mut haystack = format!(
            "{} {} {}",
            ticket.title, ticket.description, ticket.issue_type
        )
        .to_lowercase();

        for label in &ticket.labels {
            haystack.push(' ');
            haystack.push_str(&label.to_lowercase());
        }

        let security_keywords = ["cve-", "vulnerability", "security", "cwe-", "cvss"];
        let upgrade_keywords = ["upgrade", "update", "bump", "patch", "dependency"];

        let has_security_keyword = security_keywords
            .iter()
            .any(|keyword| haystack.contains(keyword));
        let has_upgrade_keyword = upgrade_keywords
            .iter()
            .any(|keyword| haystack.contains(keyword));

        has_security_keyword || (has_upgrade_keyword && haystack.contains("cve"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PlatformType;

    fn make_ticket(
        title: &str,
        description: &str,
        issue_type: &str,
        labels: Vec<&str>,
    ) -> JiraTicket {
        JiraTicket {
            id: "1".to_string(),
            key: "MAINT-1".to_string(),
            title: title.to_string(),
            description: description.to_string(),
            labels: labels.into_iter().map(|label| label.to_string()).collect(),
            status: "Open".to_string(),
            priority: "P1".to_string(),
            issue_type: issue_type.to_string(),
            assignee: "Unassigned".to_string(),
            reporter: "Reporter".to_string(),
            created: "2024-01-01".to_string(),
            updated: "2024-01-02".to_string(),
            attachments: Vec::new(),
            comments: Vec::new(),
        }
    }

    #[test]
    fn detects_security_keywords_in_ticket() {
        let ticket = make_ticket(
            "Upgrade dependency",
            "Apply CVE-2024-0001 fix",
            "Task",
            vec![],
        );
        assert!(PromptGenerator::is_security_task(&ticket));
    }

    #[test]
    fn detects_security_from_labels() {
        let ticket = make_ticket(
            "Routine maintenance",
            "No mention of cve",
            "Task",
            vec!["security"],
        );
        assert!(PromptGenerator::is_security_task(&ticket));
    }

    #[test]
    fn does_not_flag_upgrade_without_cve() {
        let ticket = make_ticket(
            "Upgrade dependencies",
            "Upgrade libraries for performance",
            "Task",
            vec![],
        );
        assert!(!PromptGenerator::is_security_task(&ticket));
    }

    #[test]
    fn unknown_platform_guidelines_empty() {
        let guidelines = PromptGenerator::get_platform_guidelines(&PlatformType::Unknown);
        assert!(guidelines.trim().is_empty());
    }

    #[test]
    fn template_override_dir_prefers_xdg_config_home() {
        let original_xdg = env::var("XDG_CONFIG_HOME").ok();
        let original_home = env::var("HOME").ok();

        let temp_dir = tempfile::tempdir().expect("tempdir");
        unsafe {
            env::set_var("XDG_CONFIG_HOME", temp_dir.path());
            env::remove_var("HOME");
        }

        let path = template_override_dir().expect("path");
        assert_eq!(path, temp_dir.path().join("repos").join("fix"));

        unsafe {
            if let Some(value) = original_xdg {
                env::set_var("XDG_CONFIG_HOME", value);
            } else {
                env::remove_var("XDG_CONFIG_HOME");
            }

            if let Some(value) = original_home {
                env::set_var("HOME", value);
            } else {
                env::remove_var("HOME");
            }
        }
    }
}
