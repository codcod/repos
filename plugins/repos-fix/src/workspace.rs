use anyhow::{Context, Result};
use repos::Repository;
use std::fs;
use std::path::{Path, PathBuf};

pub struct WorkspaceManager {
    workspace_root: PathBuf,
}

impl WorkspaceManager {
    pub fn new(workspace_root: Option<&Path>, ticket_id: String) -> Self {
        let workspace_root = workspace_root
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("workspace").join("fix").join(&ticket_id));

        Self { workspace_root }
    }

    pub fn setup(&self) -> Result<()> {
        fs::create_dir_all(&self.workspace_root).context("Failed to create workspace directory")?;
        Ok(())
    }

    pub fn get_ticket_dir(&self) -> PathBuf {
        self.workspace_root.clone()
    }
}

pub struct RepoManager<'a> {
    repo: &'a Repository,
}

impl<'a> RepoManager<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    pub fn setup_repository(&self) -> Result<PathBuf> {
        let path = self
            .repo
            .path
            .as_ref()
            .context("Repository path is missing in plugin context")?;
        let repo_dir = Path::new(path);

        if !repo_dir.exists() || !repo_dir.join(".git").exists() {
            anyhow::bail!(
                "Repository path is not a git checkout: {}",
                repo_dir.display()
            );
        }

        println!("Using repository from core context: {}", repo_dir.display());
        Ok(repo_dir.to_path_buf())
    }
}
