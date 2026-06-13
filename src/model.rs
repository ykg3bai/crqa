use std::cmp::Ordering;
use std::fmt;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    C,
    Cpp,
    Rust,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Rust => "Rust",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
            Self::Info => 2,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Info => "INFO",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub file: String,
    pub line: usize,
    pub severity: Severity,
    pub rule: String,
    pub language: Language,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileReport {
    pub path: String,
    pub language: Language,
    pub line_count: usize,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub files: usize,
    pub total_lines: usize,
    pub score: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisResult {
    pub files: Vec<FileReport>,
    pub issues: Vec<Issue>,
    pub summary: Summary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_review: Option<AiReviewResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiReviewResult {
    pub provider: String,
    pub model: String,
    pub score: u8,
    pub summary: String,
    pub recommendations: Vec<String>,
    pub raw_response: String,
}

pub fn compare_issues(left: &Issue, right: &Issue) -> Ordering {
    left.severity
        .rank()
        .cmp(&right.severity.rank())
        .then_with(|| left.file.cmp(&right.file))
        .then_with(|| left.line.cmp(&right.line))
        .then_with(|| left.rule.cmp(&right.rule))
}

pub fn issue(
    file: &str,
    language: Language,
    line: usize,
    severity: Severity,
    rule: &str,
    message: impl Into<String>,
) -> Issue {
    Issue {
        file: file.to_string(),
        line,
        severity,
        rule: rule.to_string(),
        language,
        message: message.into(),
    }
}
