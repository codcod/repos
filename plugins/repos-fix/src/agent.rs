use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
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

        let stdout_handle = child.stdout.take().map(|stdout| {
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Show progress indicators
                    Self::display_progress(&line, ask);
                }
            })
        });

        let stderr_handle = child.stderr.take().map(|stderr| {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("{}", line);
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
            anyhow::bail!("cursor-agent exited with status: {}", status);
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
}
