mod dependencies;
mod index;
mod platform;
mod structure;

pub use dependencies::{DependencyAnalyzer, DependencyInfo};
pub use index::RepoIndex;
pub use platform::{PlatformDetector, PlatformInfo};
pub use structure::{
    ArchitecturePatterns, BuildCommands, ProjectStructure, StructureAnalyzer, TestStructure,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete project analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub platform: PlatformInfo,
    pub dependencies: DependencyInfo,
    pub architecture_patterns: ArchitecturePatterns,
    pub test_structure: TestStructure,
    pub project_structure: ProjectStructure,
    pub build_commands: BuildCommands,
}

/// Main project analyzer - coordinates all analysis modules
pub struct ProjectAnalyzer {
    repo_path: std::path::PathBuf,
}

impl ProjectAnalyzer {
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Perform complete project analysis with single filesystem traversal
    pub fn analyze(&self) -> Result<ProjectAnalysis> {
        // Single pass: build the file index once
        let index = RepoIndex::build(&self.repo_path)?;

        // Detect platform
        let platform_detector = PlatformDetector::new(&index, &self.repo_path);
        let platform = platform_detector.detect();

        // Analyze dependencies
        let dependency_analyzer = DependencyAnalyzer::new(&index);
        let dependencies = dependency_analyzer.analyze(&platform.platform_type);

        // Analyze structure and patterns
        let structure_analyzer = StructureAnalyzer::new(&index, &self.repo_path);
        let architecture_patterns =
            structure_analyzer.analyze_architecture(&platform.platform_type);
        let test_structure = structure_analyzer.analyze_test_structure(&platform.platform_type);
        let project_structure =
            structure_analyzer.analyze_project_structure(&platform.platform_type);
        let build_commands = structure_analyzer.determine_build_commands(&platform.platform_type);

        Ok(ProjectAnalysis {
            platform,
            dependencies,
            architecture_patterns,
            test_structure,
            project_structure,
            build_commands,
        })
    }
}
