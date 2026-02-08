use crate::analysis::index::RepoIndex;
use crate::domain::PlatformType;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DependencyInfo {
    pub java: HashMap<String, Vec<String>>,
    pub ios: HashMap<String, Vec<String>>,
    pub android: HashMap<String, Vec<String>>,
    pub angular: HashMap<String, Vec<String>>,
}

pub struct DependencyAnalyzer<'a> {
    index: &'a RepoIndex,
}

impl<'a> DependencyAnalyzer<'a> {
    pub fn new(index: &'a RepoIndex) -> Self {
        Self { index }
    }

    pub fn analyze(&self, platform: &PlatformType) -> DependencyInfo {
        let mut deps = DependencyInfo::default();

        match platform {
            PlatformType::Java => {
                if let Some(pom_deps) = self.parse_pom_xml() {
                    deps.java.insert("maven".to_string(), pom_deps);
                }
                if let Some(gradle_deps) = self.parse_gradle_files() {
                    deps.java.insert("gradle".to_string(), gradle_deps);
                }
            }
            PlatformType::Android => {
                if let Some(gradle_deps) = self.parse_gradle_files() {
                    deps.android.insert("gradle".to_string(), gradle_deps);
                }
            }
            PlatformType::Ios => {
                if let Some(podfile_deps) = self.parse_podfile() {
                    deps.ios.insert("cocoapods".to_string(), podfile_deps);
                }
            }
            PlatformType::Angular => {
                if let Some(npm_deps) = self.parse_package_json() {
                    deps.angular.insert("npm".to_string(), npm_deps);
                }
            }
            PlatformType::Unknown => {}
        }

        deps
    }

    fn parse_pom_xml(&self) -> Option<Vec<String>> {
        let pom_files = self.index.files_with_name("pom.xml");
        if pom_files.is_empty() {
            return None;
        }

        let content = fs::read_to_string(pom_files[0]).ok()?;
        let deps: Vec<String> = content
            .lines()
            .filter(|line| line.trim().starts_with("<artifactId>"))
            .map(|line| line.trim().to_string())
            .collect();

        if deps.is_empty() { None } else { Some(deps) }
    }

    fn parse_gradle_files(&self) -> Option<Vec<String>> {
        let mut all_deps = Vec::new();

        let gradle_files: Vec<_> = self
            .index
            .files_with_name("build.gradle")
            .into_iter()
            .chain(self.index.files_with_name("build.gradle.kts"))
            .collect();

        for gradle_file in gradle_files {
            if let Ok(content) = fs::read_to_string(gradle_file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("implementation")
                        || trimmed.contains("api")
                        || trimmed.contains("testImplementation")
                    {
                        all_deps.push(trimmed.to_string());
                    }
                }
            }
        }

        if all_deps.is_empty() {
            None
        } else {
            Some(all_deps)
        }
    }

    fn parse_podfile(&self) -> Option<Vec<String>> {
        let podfiles = self.index.files_with_name("Podfile");
        if podfiles.is_empty() {
            return None;
        }

        let content = fs::read_to_string(podfiles[0]).ok()?;
        let pods: Vec<String> = content
            .lines()
            .filter(|line| line.trim().starts_with("pod "))
            .map(|line| line.trim().to_string())
            .collect();

        if pods.is_empty() { None } else { Some(pods) }
    }

    fn parse_package_json(&self) -> Option<Vec<String>> {
        let package_files = self.index.files_with_name("package.json");
        if package_files.is_empty() {
            return None;
        }

        let content = fs::read_to_string(package_files[0]).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        let mut deps = Vec::new();

        if let Some(dependencies) = json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version) in dependencies {
                deps.push(format!("{}: {}", name, version));
            }
        }

        if deps.is_empty() { None } else { Some(deps) }
    }
}
