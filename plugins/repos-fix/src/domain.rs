use serde::{Deserialize, Serialize};
use std::fmt;

/// Platform types supported by the analyzer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    Ios,
    Android,
    Angular,
    Java,
    Unknown,
}

impl PlatformType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
            Self::Angular => "angular",
            Self::Java => "java",
            Self::Unknown => "unknown",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Ios => "📱",
            Self::Android => "🤖",
            Self::Angular => "🌐",
            Self::Java => "☕",
            Self::Unknown => "💻",
        }
    }
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Programming languages detected in the codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Swift,
    #[serde(rename = "objective-c")]
    ObjectiveC,
    Kotlin,
    Java,
    TypeScript,
    JavaScript,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Swift => "swift",
            Self::ObjectiveC => "objective-c",
            Self::Kotlin => "kotlin",
            Self::Java => "java",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Build tool/framework detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    CocoaPods,
    SwiftPackageManager,
    Gradle,
    Maven,
    Npm,
    Yarn,
}

impl Framework {
    pub fn as_str(&self) -> &str {
        match self {
            Self::CocoaPods => "cocoapods",
            Self::SwiftPackageManager => "swift-package-manager",
            Self::Gradle => "gradle",
            Self::Maven => "maven",
            Self::Npm => "npm",
            Self::Yarn => "yarn",
        }
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Dependency injection frameworks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum DiFramework {
    Koin,
    Hilt,
    Dagger,
    Spring,
}

impl DiFramework {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Koin => "koin",
            Self::Hilt => "hilt",
            Self::Dagger => "dagger",
            Self::Spring => "spring",
        }
    }
}

impl fmt::Display for DiFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Reactive programming frameworks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ReactiveFramework {
    RxSwift,
    RxJava,
    Combine,
    Coroutines,
    RxJS,
}

impl ReactiveFramework {
    pub fn as_str(&self) -> &str {
        match self {
            Self::RxSwift => "rxswift",
            Self::RxJava => "rxjava",
            Self::Combine => "combine",
            Self::Coroutines => "coroutines",
            Self::RxJS => "rxjs",
        }
    }
}

impl fmt::Display for ReactiveFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// UI frameworks detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum UiFramework {
    SwiftUI,
    UIKit,
    #[serde(rename = "jetpack-compose")]
    JetpackCompose,
    Angular,
}

impl UiFramework {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SwiftUI => "swiftui",
            Self::UIKit => "uikit",
            Self::JetpackCompose => "jetpack-compose",
            Self::Angular => "angular",
        }
    }
}

impl fmt::Display for UiFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Test frameworks detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestFramework {
    JUnit,
    Mockito,
    MockK,
    XCTest,
    Quick,
    Jasmine,
    Jest,
}

impl TestFramework {
    pub fn as_str(&self) -> &str {
        match self {
            Self::JUnit => "junit",
            Self::Mockito => "mockito",
            Self::MockK => "mockk",
            Self::XCTest => "xctest",
            Self::Quick => "quick",
            Self::Jasmine => "jasmine",
            Self::Jest => "jest",
        }
    }
}

impl fmt::Display for TestFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
