use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct CursorAgentRunner {
    api_key: String,
}

impl CursorAgentRunner {
    pub fn new() -> Result<Self> {
        let api_key =
            env::var("CURSOR_API_KEY").context("CURSOR_API_KEY environment variable not set")?;

        // Check if cursor-agent is available
        Self::check_cursor_agent()?;

        Ok(Self { api_key })
    }

    fn check_cursor_agent() -> Result<()> {
        let output = Command::new("cursor-agent").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("Found cursor-agent: {}", version.trim());
                Ok(())
            }
            _ => {
                anyhow::bail!(
                    "cursor-agent not found. Please install it:\n\
                     curl https://cursor.com/install -fsS | bash"
                );
            }
        }
    }

    pub fn run(&self, workspace_dir: &Path, prompt: &str, ask: bool) -> Result<()> {
        println!("\n{}", "=".repeat(60));
        if ask {
            println!("🚀 Starting cursor-agent in ASK mode");
            println!(
                "🔍 No code will be changed - only analyzing and creating solution proposal..."
            );
        } else {
            println!("🚀 Starting cursor-agent");
            println!("💭 This may take several minutes while the AI analyzes and codes...");
        }
        println!("{}", "=".repeat(60));
        println!();

        let mut cmd = Command::new("cursor-agent");
        cmd.arg("--api-key")
            .arg(&self.api_key)
            .arg("--print")
            .arg("--force")
            .arg(prompt)
            .current_dir(workspace_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn cursor-agent")?;
        let stdout_tail = Arc::new(Mutex::new(Vec::new()));
        let stderr_tail = Arc::new(Mutex::new(Vec::new()));

        let stdout_handle = child.stdout.take().map(|stdout| {
            let stdout_tail = Arc::clone(&stdout_tail);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Show progress indicators
                    Self::display_progress(&line, ask);
                    Self::capture_tail_line(&stdout_tail, line);
                }
            })
        });

        let stderr_handle = child.stderr.take().map(|stderr| {
            let stderr_tail = Arc::clone(&stderr_tail);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("{}", line);
                    Self::capture_tail_line(&stderr_tail, line);
                }
            })
        });

        let status = child.wait().context("Failed to wait for cursor-agent")?;

        if let Some(handle) = stdout_handle {
            let _ = handle.join();
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        println!();
        println!("{}", "=".repeat(60));

        if status.success() {
            if ask {
                println!("🎉 Solution analysis completed successfully!");
                println!("📄 SOLUTION_SUMMARY.md should be created with the proposed solution");
            } else {
                println!("🎉 Code fix implementation completed successfully!");
                println!("📄 Check SOLUTION_SUMMARY.md for details");
            }
        } else {
            let stdout_tail = stdout_tail
                .lock()
                .map(|lines| lines.clone())
                .unwrap_or_default();
            let stderr_tail = stderr_tail
                .lock()
                .map(|lines| lines.clone())
                .unwrap_or_default();
            let mut tail_summary = String::new();

            if !stdout_tail.is_empty() {
                tail_summary.push_str("\n--- stdout (tail) ---\n");
                tail_summary.push_str(&stdout_tail.join("\n"));
            }
            if !stderr_tail.is_empty() {
                tail_summary.push_str("\n--- stderr (tail) ---\n");
                tail_summary.push_str(&stderr_tail.join("\n"));
            }

            anyhow::bail!(
                "cursor-agent exited with status: {}{}",
                status,
                tail_summary
            );
        }

        println!("{}", "=".repeat(60));
        println!();

        Ok(())
    }

    fn display_progress(line: &str, ask: bool) {
        let line_lower = line.to_lowercase();

        // Simple progress indicators based on keywords
        if line_lower.contains("analyzing") || line_lower.contains("reading") {
            print!("🔍 Analyzing... ");
        } else if line_lower.contains("planning") || line_lower.contains("thinking") {
            print!("💡 Planning... ");
        } else if !ask && (line_lower.contains("writing") || line_lower.contains("creating")) {
            print!("⚡ Implementing... ");
        } else if line_lower.contains("testing") || line_lower.contains("building") {
            print!("✅ Validating... ");
        } else if line_lower.contains("error") || line_lower.contains("failed") {
            eprintln!("❌ Error: {}", line);
        }
    }

    fn capture_tail_line(buffer: &Arc<Mutex<Vec<String>>>, line: String) {
        const MAX_LINES: usize = 80;
        if let Ok(mut lines) = buffer.lock() {
            if lines.len() >= MAX_LINES {
                let overflow = lines.len() + 1 - MAX_LINES;
                lines.drain(0..overflow);
            }
            lines.push(line);
        }
    }

    pub fn run_with_retry(
        &self,
        workspace_dir: &Path,
        prompt: &str,
        ask: bool,
        max_retries: u32,
    ) -> Result<()> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            let current_prompt = if attempt == 1 {
                prompt.to_string()
            } else {
                let error_message = last_error
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown error".to_string());
                format!(
                    "{}\n\n**PREVIOUS ATTEMPT FAILED**\nError: {}\n\
                    Please analyze the error and fix the code accordingly.",
                    prompt, error_message
                )
            };

            match self.run(workspace_dir, &current_prompt, ask) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        eprintln!("\n⚠️  Attempt {} failed", attempt);
                        eprintln!("🔄 Retrying... ({}/{})\n", attempt + 1, max_retries);
                    }
                }
            }
        }

        if let Some(error) = last_error {
            anyhow::bail!(
                "Failed after {} attempts. Last error: {}",
                max_retries,
                error
            );
        }

        anyhow::bail!("Failed after {} attempts.", max_retries);
    }

    pub fn verify_solution(&self, workspace_dir: &Path) -> Result<bool> {
        if !self.verify_analysis(workspace_dir)? {
            return Ok(false);
        }

        let solution_file = workspace_dir.join("SOLUTION_SUMMARY.md");

        if !solution_file.exists() {
            eprintln!("⚠️  SOLUTION_SUMMARY.md not found");
            return Ok(false);
        }

        let content =
            fs::read_to_string(&solution_file).context("Failed to read SOLUTION_SUMMARY.md")?;

        if content.trim().is_empty() {
            eprintln!("⚠️  SOLUTION_SUMMARY.md is empty");
            return Ok(false);
        }

        println!("✅ SOLUTION_SUMMARY.md created successfully");
        Ok(true)
    }

    fn verify_analysis(&self, workspace_dir: &Path) -> Result<bool> {
        let analysis_file = workspace_dir.join("ANALYSIS.md");

        if !analysis_file.exists() {
            eprintln!("⚠️  ANALYSIS.md not found");
            return Ok(false);
        }

        let content = fs::read_to_string(&analysis_file).context("Failed to read ANALYSIS.md")?;

        if content.trim().is_empty() {
            eprintln!("⚠️  ANALYSIS.md is empty");
            return Ok(false);
        }

        let required_sections = [
            "- Root cause hypothesis:",
            "- Target files/components:",
            "- Plan:",
        ];
        let lines: Vec<&str> = content.lines().collect();
        let mut all_sections_present = true;

        for section in required_sections {
            let mut found = false;
            let mut filled = false;

            for (index, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if let Some(remainder) = trimmed.strip_prefix(section) {
                    found = true;
                    let remainder = remainder.trim();
                    if !remainder.is_empty() {
                        filled = true;
                        break;
                    }

                    for next_line in lines.iter().skip(index + 1) {
                        let next_trim = next_line.trim();
                        if next_trim.is_empty() {
                            continue;
                        }
                        if required_sections
                            .iter()
                            .any(|label| next_trim.starts_with(label))
                        {
                            break;
                        }
                        filled = true;
                        break;
                    }
                    break;
                }
            }

            if !found {
                eprintln!("⚠️  ANALYSIS.md missing section: {}", section);
                all_sections_present = false;
                continue;
            }

            if !filled {
                eprintln!("⚠️  ANALYSIS.md section not filled: {}", section);
                all_sections_present = false;
            }
        }

        if !all_sections_present {
            return Ok(false);
        }

        println!("✅ ANALYSIS.md created successfully");
        Ok(true)
    }
}
