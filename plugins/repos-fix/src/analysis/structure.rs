use crate::analysis::index::RepoIndex;
use crate::domain::{PlatformType, TestFramework};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ArchitecturePatterns {
    pub dependency_injection: Vec<String>,
    pub reactive: Vec<String>,
    pub ui_framework: Vec<String>,
    pub architecture: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestStructure {
    pub test_directories: Vec<String>,
    pub test_frameworks: Vec<TestFramework>,
    pub test_patterns: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectStructure {
    pub source_directories: Vec<String>,
    pub resource_directories: Vec<String>,
    pub config_files: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildCommands {
    pub main_build: String,
    pub test_compile: Option<String>,
    pub test_run: String,
}

pub struct StructureAnalyzer<'a> {
    index: &'a RepoIndex,
    root: &'a Path,
}

impl<'a> StructureAnalyzer<'a> {
    pub fn new(index: &'a RepoIndex, root: &'a Path) -> Self {
        Self { index, root }
    }

    pub fn analyze_architecture(&self, platform: &PlatformType) -> ArchitecturePatterns {
        let mut patterns = ArchitecturePatterns::default();

        let source_files = self.collect_source_files(platform);

        // Sample first 50 files to detect patterns
        for file_path in source_files.iter().take(50) {
            if let Ok(content) = fs::read_to_string(file_path) {
                self.detect_di_patterns(&content, &mut patterns);
                self.detect_reactive_patterns(&content, &mut patterns);
                self.detect_ui_patterns(&content, platform, &mut patterns);
            }
        }

        patterns
    }

    fn detect_di_patterns(&self, content: &str, patterns: &mut ArchitecturePatterns) {
        if content.contains("import Koin") || content.contains("org.koin") {
            Self::add_unique(&mut patterns.dependency_injection, "koin");
        }
        if content.contains("import Hilt") || content.contains("dagger.hilt") {
            Self::add_unique(&mut patterns.dependency_injection, "hilt");
        }
        if content.contains("import Dagger") || content.contains("dagger.") {
            Self::add_unique(&mut patterns.dependency_injection, "dagger");
        }
        if content.contains("@Inject") || content.contains("@Autowired") {
            Self::add_unique(&mut patterns.dependency_injection, "spring");
        }
    }

    fn detect_reactive_patterns(&self, content: &str, patterns: &mut ArchitecturePatterns) {
        if content.contains("import RxSwift") || content.contains("import RxCocoa") {
            Self::add_unique(&mut patterns.reactive, "rxswift");
        }
        if content.contains("import RxJava") || content.contains("io.reactivex") {
            Self::add_unique(&mut patterns.reactive, "rxjava");
        }
        if content.contains("import Combine") {
            Self::add_unique(&mut patterns.reactive, "combine");
        }
        if content.contains("kotlinx.coroutines") {
            Self::add_unique(&mut patterns.reactive, "coroutines");
        }
        if content.contains("import { Observable }") || content.contains("rxjs") {
            Self::add_unique(&mut patterns.reactive, "rxjs");
        }
    }

    fn detect_ui_patterns(
        &self,
        content: &str,
        platform: &PlatformType,
        patterns: &mut ArchitecturePatterns,
    ) {
        if content.contains("import SwiftUI") {
            Self::add_unique(&mut patterns.ui_framework, "swiftui");
        }
        if content.contains("import UIKit") {
            Self::add_unique(&mut patterns.ui_framework, "uikit");
        }
        if content.contains("androidx.compose") {
            Self::add_unique(&mut patterns.ui_framework, "jetpack-compose");
        }
        if content.contains("@Component") && *platform == PlatformType::Angular {
            Self::add_unique(&mut patterns.ui_framework, "angular");
        }
    }

    pub fn analyze_test_structure(&self, platform: &PlatformType) -> TestStructure {
        let test_directories = self.find_test_directories();
        let test_frameworks = self.detect_test_frameworks(platform);
        let test_patterns = self.determine_test_patterns(platform);

        TestStructure {
            test_directories,
            test_frameworks,
            test_patterns,
        }
    }

    fn find_test_directories(&self) -> Vec<String> {
        let test_dir_names = ["test", "tests", "Test", "Tests", "androidTest", "unitTest"];
        let mut test_dirs = BTreeSet::new();

        for file_path in &self.index.files {
            if let Some(parent) = file_path.parent()
                && let Some(name) = parent.file_name().and_then(|n| n.to_str())
                && test_dir_names.contains(&name)
                && let Ok(rel_path) = parent.strip_prefix(self.root)
            {
                let rel_str = rel_path.to_string_lossy().to_string();
                test_dirs.insert(rel_str);
            }
        }

        test_dirs.into_iter().collect()
    }

    fn detect_test_frameworks(&self, platform: &PlatformType) -> Vec<TestFramework> {
        let mut frameworks = HashSet::new();
        let test_files = self.find_test_files(platform);

        for test_file in test_files.iter().take(20) {
            if let Ok(content) = fs::read_to_string(test_file) {
                match platform {
                    PlatformType::Java | PlatformType::Android => {
                        if content.contains("import org.junit") {
                            frameworks.insert(TestFramework::JUnit);
                        }
                        if content.contains("import org.mockito") {
                            frameworks.insert(TestFramework::Mockito);
                        }
                        if content.contains("import io.mockk") {
                            frameworks.insert(TestFramework::MockK);
                        }
                    }
                    PlatformType::Ios => {
                        if content.contains("import XCTest") {
                            frameworks.insert(TestFramework::XCTest);
                        }
                        if content.contains("import Quick") {
                            frameworks.insert(TestFramework::Quick);
                        }
                    }
                    PlatformType::Angular => {
                        if content.contains("jasmine") || content.contains("describe(") {
                            frameworks.insert(TestFramework::Jasmine);
                        }
                        if content.contains("jest") {
                            frameworks.insert(TestFramework::Jest);
                        }
                    }
                    PlatformType::Unknown => {}
                }
            }
        }

        frameworks.into_iter().collect()
    }

    fn find_test_files(&self, platform: &PlatformType) -> Vec<&Path> {
        let patterns: Vec<&str> = match platform {
            PlatformType::Java | PlatformType::Android => {
                vec!["Test.java", "Test.kt", "Tests.java", "Tests.kt"]
            }
            PlatformType::Ios => vec!["Test.swift", "Tests.swift", "Spec.swift"],
            PlatformType::Angular => vec![".spec.ts", ".spec.js", "test.ts"],
            PlatformType::Unknown => vec![],
        };

        self.index
            .files
            .iter()
            .filter(|path| {
                let path_str = path.to_string_lossy();
                patterns.iter().any(|pattern| path_str.contains(pattern))
            })
            .map(|p| p.as_path())
            .collect()
    }

    fn determine_test_patterns(&self, platform: &PlatformType) -> Vec<String> {
        let mut patterns = vec!["unit-tests".to_string()];
        if matches!(platform, PlatformType::Android | PlatformType::Ios) {
            patterns.push("ui-tests".to_string());
        }
        patterns
    }

    pub fn analyze_project_structure(&self, platform: &PlatformType) -> ProjectStructure {
        let source_dirs = self.find_source_directories(platform);
        let resource_dirs = self.find_resource_directories(platform);
        let config_files = self.find_config_files(platform);

        ProjectStructure {
            source_directories: source_dirs,
            resource_directories: resource_dirs,
            config_files,
        }
    }

    fn find_source_directories(&self, platform: &PlatformType) -> Vec<String> {
        let patterns: Vec<&str> = match platform {
            PlatformType::Java | PlatformType::Android => {
                vec!["src/main/java", "src/main/kotlin", "src"]
            }
            PlatformType::Ios => vec!["Sources", "src"],
            PlatformType::Angular => vec!["src/app", "src"],
            PlatformType::Unknown => vec![],
        };

        self.find_matching_dirs(&patterns)
    }

    fn find_resource_directories(&self, platform: &PlatformType) -> Vec<String> {
        let patterns: Vec<&str> = match platform {
            PlatformType::Java | PlatformType::Android => vec!["src/main/resources", "res"],
            PlatformType::Ios => vec!["Resources", "Assets.xcassets"],
            PlatformType::Angular => vec!["src/assets"],
            PlatformType::Unknown => vec![],
        };

        self.find_matching_dirs(&patterns)
    }

    fn find_config_files(&self, platform: &PlatformType) -> Vec<String> {
        let names: Vec<&str> = match platform {
            PlatformType::Java | PlatformType::Android => {
                vec!["build.gradle", "pom.xml", "settings.gradle"]
            }
            PlatformType::Ios => vec!["Package.swift", "Podfile", "project.pbxproj"],
            PlatformType::Angular => vec!["angular.json", "package.json", "tsconfig.json"],
            PlatformType::Unknown => vec![],
        };

        let mut config_files = Vec::new();
        for name in names {
            for file in self.index.files_with_name(name) {
                if let Ok(rel_path) = file.strip_prefix(self.root) {
                    config_files.push(rel_path.to_string_lossy().to_string());
                }
            }
        }

        config_files
    }

    fn find_matching_dirs(&self, patterns: &[&str]) -> Vec<String> {
        let mut result = BTreeSet::new();

        for file_path in &self.index.files {
            let path_str = file_path.to_string_lossy();
            for pattern in patterns {
                if path_str.contains(pattern) {
                    if let Some(parent) = file_path.parent()
                        && let Ok(rel_path) = parent.strip_prefix(self.root)
                    {
                        result.insert(rel_path.to_string_lossy().to_string());
                    }
                    break;
                }
            }
        }

        result.into_iter().collect()
    }

    pub fn determine_build_commands(&self, platform: &PlatformType) -> BuildCommands {
        match platform {
            PlatformType::Java => {
                if self.index.has_file("pom.xml") {
                    BuildCommands {
                        main_build: "mvn compile".to_string(),
                        test_compile: Some("mvn test-compile".to_string()),
                        test_run: "mvn test".to_string(),
                    }
                } else {
                    BuildCommands {
                        main_build: "./gradlew build".to_string(),
                        test_compile: Some("./gradlew testClasses".to_string()),
                        test_run: "./gradlew test".to_string(),
                    }
                }
            }
            PlatformType::Android => BuildCommands {
                main_build: "./gradlew assembleDebug".to_string(),
                test_compile: Some("./gradlew compileDebugUnitTestKotlin".to_string()),
                test_run: "./gradlew testDebugUnitTest".to_string(),
            },
            PlatformType::Ios => BuildCommands {
                main_build: "xcodebuild -scheme <scheme> build".to_string(),
                test_compile: Some("xcodebuild -scheme <scheme> build-for-testing".to_string()),
                test_run: "xcodebuild test -scheme <scheme>".to_string(),
            },
            PlatformType::Angular => BuildCommands {
                main_build: "npm run build".to_string(),
                test_compile: None,
                test_run: "npm test".to_string(),
            },
            PlatformType::Unknown => BuildCommands {
                main_build: "make".to_string(),
                test_compile: None,
                test_run: "make test".to_string(),
            },
        }
    }

    fn collect_source_files(&self, platform: &PlatformType) -> Vec<&Path> {
        let extensions: Vec<&str> = match platform {
            PlatformType::Java => vec!["java", "kt"],
            PlatformType::Android => vec!["java", "kt"],
            PlatformType::Ios => vec!["swift", "m", "h"],
            PlatformType::Angular => vec!["ts", "js"],
            PlatformType::Unknown => vec![],
        };

        self.index
            .files_with_extensions(&extensions)
            .into_iter()
            .map(|p| p.as_path())
            .collect()
    }

    fn add_unique(vec: &mut Vec<String>, item: &str) {
        if !vec.iter().any(|existing| existing == item) {
            vec.push(item.to_string());
        }
    }
}
