use crate::analysis::index::RepoIndex;
use crate::domain::{Framework, Language, PlatformType};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformInfo {
    pub platform_type: PlatformType,
    pub languages: Vec<Language>,
    pub frameworks: Vec<Framework>,
}

pub struct PlatformDetector<'a> {
    index: &'a RepoIndex,
    root: &'a Path,
}

impl<'a> PlatformDetector<'a> {
    pub fn new(index: &'a RepoIndex, root: &'a Path) -> Self {
        Self { index, root }
    }

    pub fn detect(&self) -> PlatformInfo {
        let platform_type = self.detect_platform_type();
        let languages = self.detect_languages(&platform_type);
        let frameworks = self.detect_frameworks(&platform_type);

        PlatformInfo {
            platform_type,
            languages,
            frameworks,
        }
    }

    fn detect_platform_type(&self) -> PlatformType {
        // iOS Detection
        if self.index.has_path_pattern(".xcodeproj") || self.index.has_path_pattern(".xcworkspace")
        {
            return PlatformType::Ios;
        }

        // Android Detection
        if self.index.has_path_pattern("AndroidManifest.xml")
            || self.root.join("app/src/main/AndroidManifest.xml").exists()
            || self.has_android_gradle_plugin()
        {
            return PlatformType::Android;
        }

        // Angular Detection
        if self.index.has_file("angular.json")
            || (self.index.has_file("package.json") && self.has_angular_in_package_json())
        {
            return PlatformType::Angular;
        }

        // Java Backend Detection
        if self.index.has_file("pom.xml") || self.index.has_file("build.gradle") {
            return PlatformType::Java;
        }

        PlatformType::Unknown
    }

    fn detect_languages(&self, platform: &PlatformType) -> Vec<Language> {
        let mut languages = Vec::new();

        match platform {
            PlatformType::Ios => {
                if self.index.has_extension("swift") {
                    languages.push(Language::Swift);
                }
                if self.index.has_extension("m") || self.index.has_extension("h") {
                    languages.push(Language::ObjectiveC);
                }
            }
            PlatformType::Android | PlatformType::Java => {
                if self.index.has_extension("kt") {
                    languages.push(Language::Kotlin);
                }
                if self.index.has_extension("java") {
                    languages.push(Language::Java);
                }
            }
            PlatformType::Angular => {
                languages.push(Language::TypeScript);
                if self.index.has_extension("js") {
                    languages.push(Language::JavaScript);
                }
            }
            PlatformType::Unknown => {}
        }

        languages
    }

    fn detect_frameworks(&self, platform: &PlatformType) -> Vec<Framework> {
        let mut frameworks = Vec::new();

        match platform {
            PlatformType::Ios => {
                if self.index.has_file("Podfile") {
                    frameworks.push(Framework::CocoaPods);
                }
                if self.index.has_file("Package.swift") {
                    frameworks.push(Framework::SwiftPackageManager);
                }
            }
            PlatformType::Android => {
                frameworks.push(Framework::Gradle);
            }
            PlatformType::Java => {
                if self.index.has_file("pom.xml") {
                    frameworks.push(Framework::Maven);
                }
                if self.index.has_file("build.gradle") || self.index.has_file("build.gradle.kts") {
                    frameworks.push(Framework::Gradle);
                }
            }
            PlatformType::Angular => {
                if self.index.has_file("package.json") {
                    frameworks.push(Framework::Npm);
                }
                if self.index.has_file("yarn.lock") {
                    frameworks.push(Framework::Yarn);
                }
            }
            PlatformType::Unknown => {}
        }

        frameworks
    }

    fn has_android_gradle_plugin(&self) -> bool {
        let gradle_files = self
            .index
            .files_with_name("build.gradle")
            .into_iter()
            .chain(self.index.files_with_name("build.gradle.kts"));

        for gradle_file in gradle_files {
            if let Ok(content) = fs::read_to_string(gradle_file)
                && (content.contains("com.android.application")
                    || content.contains("com.android.library")
                    || content.contains("com.android.test"))
            {
                return true;
            }
        }

        false
    }

    fn has_angular_in_package_json(&self) -> bool {
        if let Some(package_json) = self.index.files_with_name("package.json").first()
            && let Ok(content) = std::fs::read_to_string(package_json)
        {
            return content.contains("@angular/core") || content.contains("@angular/cli");
        }
        false
    }
}
