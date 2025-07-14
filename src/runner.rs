//! Command execution runner for managing operations across multiple repositories

use crate::config::Repository;
use crate::git::Logger;
use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct CommandRunner {
    logger: Logger,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run_command(
        &self,
        repo: &Repository,
        command: &str,
        log_dir: Option<&str>,
    ) -> Result<()> {
        let repo_dir = repo.get_target_dir();

        // Check if directory exists
        if !Path::new(&repo_dir).exists() {
            anyhow::bail!("Repository directory does not exist: {}", repo_dir);
        }

        // Prepare log file if log directory is specified
        let log_file = if let Some(log_dir) = log_dir {
            Some(self.prepare_log_file(repo, log_dir, command, &repo_dir)?)
        } else {
            None
        };

        self.logger.info(repo, &format!("Running '{command}'"));

        // Execute command
        let mut cmd = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&repo_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = cmd.stdout.take().unwrap();
        let stderr = cmd.stderr.take().unwrap();

        let log_file = Arc::new(Mutex::new(log_file));
        let repo_name = repo.name.clone();

        // Handle stdout
        let stdout_log_file = Arc::clone(&log_file);
        let stdout_repo_name = repo_name.clone();
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            // Note: We explicitly handle Result instead of using .flatten()
            // to avoid infinite loops on repeated I/O errors
            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Print to console with colored repo name
                    println!("{} | {line}", stdout_repo_name.cyan());

                    // Write to log file if available
                    if let Some(ref mut log_file) = *stdout_log_file.lock().await {
                        writeln!(log_file, "{stdout_repo_name} | {line}").ok();
                        log_file.flush().ok();
                    }
                }
            }
        });

        // Handle stderr
        let stderr_log_file = Arc::clone(&log_file);
        let stderr_repo_name = repo_name.clone();
        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut header_written = false;

            // Note: We explicitly handle Result instead of using .flatten()
            // to avoid infinite loops on repeated I/O errors
            #[allow(clippy::manual_flatten)]
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Print to console with colored repo name
                    eprintln!("{} | {line}", stderr_repo_name.red().bold());

                    // Write to log file if available
                    if let Some(ref mut log_file) = *stderr_log_file.lock().await {
                        if !header_written {
                            writeln!(log_file, "\n=== STDERR ===").ok();
                            header_written = true;
                        }
                        writeln!(log_file, "{stderr_repo_name} | {line}").ok();
                        log_file.flush().ok();
                    }
                }
            }
        });

        // Wait for output processing to complete
        let _ = tokio::join!(stdout_handle, stderr_handle);

        // Wait for command to complete
        let status = cmd.wait()?;

        if !status.success() {
            anyhow::bail!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }

    fn prepare_log_file(
        &self,
        repo: &Repository,
        log_dir: &str,
        command: &str,
        repo_dir: &str,
    ) -> Result<File> {
        // Create log directory if it doesn't exist
        create_dir_all(log_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let log_file_path = format!("{}/{}_{}.log", log_dir, repo.name, timestamp);

        let mut log_file = File::create(&log_file_path)?;

        // Write header information
        writeln!(log_file, "Repository: {}", repo.name)?;
        writeln!(log_file, "Command: {command}")?;
        writeln!(log_file, "Directory: {repo_dir}")?;
        writeln!(log_file, "Timestamp: {}", Utc::now().to_rfc3339())?;
        writeln!(log_file, "\n=== STDOUT ===")?;

        Ok(log_file)
    }
}
