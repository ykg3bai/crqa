pub mod aireview;
pub mod config;
pub mod discovery;
pub mod model;
pub mod parser;
pub mod report;
pub mod rules;

use std::fs;
use std::io;

pub use aireview::run_ai_review;
pub use config::{Config, Target, parse_args, print_usage};
pub use model::{AiReviewResult, AnalysisResult, FileReport, Issue, Language, Severity, Summary};
pub use report::build_report;

pub fn run(config: &Config) -> io::Result<AnalysisResult> {
    let files = discovery::collect_files(&config.targets)?;
    let mut file_reports = Vec::new();
    let mut all_issues = Vec::new();
    let mut total_lines = 0;

    for path in files {
        let language = Language::from_path(&path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported source file: {}", path.display()),
            )
        })?;
        let source = fs::read_to_string(&path)?;
        let path_label = path.display().to_string();
        let report = rules::analyze_source(&path_label, language, &source);
        total_lines += report.line_count;
        all_issues.extend(report.issues.iter().cloned());
        file_reports.push(report);
    }

    all_issues.sort_by(model::compare_issues);

    for report in &mut file_reports {
        report.issues.sort_by(model::compare_issues);
    }

    let summary = build_summary(&all_issues, file_reports.len(), total_lines);
    Ok(AnalysisResult {
        files: file_reports,
        issues: all_issues,
        summary,
        ai_review: None,
    })
}

pub fn build_summary(issues: &[Issue], files: usize, total_lines: usize) -> Summary {
    let errors = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Warning)
        .count();
    let infos = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Info)
        .count();
    let penalty = issues.iter().map(issue_penalty).sum::<usize>();
    let score = 100usize.saturating_sub(penalty) as u8;

    Summary {
        errors,
        warnings,
        infos,
        files,
        total_lines,
        score,
    }
}

pub fn issue_penalty(issue: &Issue) -> usize {
    match issue.rule.as_str() {
        rules::RULE_ASSIGNMENT_IN_CONDITION => 16,
        rules::RULE_EMPTY_CONTROL_STATEMENT | rules::RULE_EQUALITY_AS_STATEMENT => 14,
        rules::RULE_VOID_MAIN_SIGNATURE => 12,
        rules::RULE_FUNCTION_BRACE_NEWLINE => 8,
        rules::RULE_CONTROL_KEYWORD_SPACE => 6,

        rules::RULE_RUST_TRANSMUTE => 10,
        rules::RULE_C_ALLOC_WITHOUT_FREE => 9,
        rules::RULE_C_DANGEROUS_API
        | rules::RULE_C_UNCHECKED_RESOURCE
        | rules::RULE_RUST_UNSAFE => 8,
        rules::RULE_CPP_RAW_NEW_DELETE => 7,
        rules::RULE_C_MACRO_SAFETY
        | rules::RULE_CPP_CATCH_ALL
        | rules::RULE_CPP_HEADER_USING_NAMESPACE
        | rules::RULE_RUST_PANIC_UNWRAP => 6,
        rules::RULE_DEEP_NESTING => 5,
        rules::RULE_C_GOTO_USAGE
        | rules::RULE_C_HEADER_GUARD
        | rules::RULE_CPP_TOO_MANY_INCLUDES => 4,
        rules::RULE_RUST_UNSAFE_WITHOUT_SAFETY => 3,
        rules::RULE_UNUSED_INCLUDE
        | rules::RULE_UNUSED_VARIABLE
        | rules::RULE_CPP_C_STYLE_CAST
        | rules::RULE_CPP_USING_NAMESPACE
        | rules::RULE_RUST_AS_CAST
        | rules::RULE_RUST_COMPLEX_TYPE
        | rules::RULE_RUST_LARGE_IMPL => 2,
        rules::RULE_CPP_NULL_LITERAL
        | rules::RULE_CPP_STD_ENDL
        | rules::RULE_RUST_ALLOW_ATTR
        | rules::RULE_RUST_CLONE_NOISE
        | rules::RULE_RUST_DBG_MACRO => 1,
        _ => match issue.severity {
            Severity::Error => 12,
            Severity::Warning => 5,
            Severity::Info => 2,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn rules(report: &FileReport) -> BTreeSet<String> {
        report
            .issues
            .iter()
            .map(|issue| issue.rule.clone())
            .collect()
    }

    #[test]
    fn detects_core_and_advanced_c_rules() {
        let source = r#"
#include <stdio.h>
#include <math.h>

#define SET_VALUE(x) { x = 1; }

void main() {
    int unused = 0;
    int used = 1;
    char name[8];
    if (used = 2);
    used == 3;
    for (int i = 0; i < 2; i++) { used++; }
    if(used) used++;
    scanf("%s", name);
    strcpy(name, "too long");
    malloc(32);
    goto done;
    if (used) {
        while (used) {
            if (used) {
                if (used) {
                    if (used) {
                        used++;
                    }
                }
            }
        }
    }
done:
}
"#;

        let report = rules::analyze_source("bad.c", Language::C, source);
        let rules = rules(&report);
        assert!(rules.contains(rules::RULE_FUNCTION_BRACE_NEWLINE));
        assert!(rules.contains(rules::RULE_SINGLE_LINE_FOR_BRACES));
        assert!(rules.contains(rules::RULE_VOID_MAIN_SIGNATURE));
        assert!(rules.contains(rules::RULE_UNUSED_INCLUDE));
        assert!(rules.contains(rules::RULE_UNUSED_VARIABLE));
        assert!(rules.contains(rules::RULE_ASSIGNMENT_IN_CONDITION));
        assert!(rules.contains(rules::RULE_EQUALITY_AS_STATEMENT));
        assert!(rules.contains(rules::RULE_EMPTY_CONTROL_STATEMENT));
        assert!(rules.contains(rules::RULE_DEEP_NESTING));
        assert!(rules.contains(rules::RULE_CONTROL_KEYWORD_SPACE));
        assert!(rules.contains(rules::RULE_C_DANGEROUS_API));
        assert!(rules.contains(rules::RULE_C_MACRO_SAFETY));
        assert!(rules.contains(rules::RULE_C_UNCHECKED_RESOURCE));
        assert!(rules.contains(rules::RULE_C_GOTO_USAGE));
        assert!(rules.contains(rules::RULE_C_ALLOC_WITHOUT_FREE));
    }

    #[test]
    fn detects_c_header_without_guard() {
        let source = "int helper(void);\n";
        let report = rules::analyze_source("missing_guard.h", Language::C, source);
        assert!(rules(&report).contains(rules::RULE_C_HEADER_GUARD));
    }

    #[test]
    fn ignores_comments_and_strings() {
        let source = r#"
#include <stdio.h>

int main(void)
{
    printf("if (x = 1); void main() {");
    // if (x = 1);
    return 0;
}
"#;
        let report = rules::analyze_source("ok.c", Language::C, source);
        let rules = rules(&report);
        assert!(!rules.contains(rules::RULE_ASSIGNMENT_IN_CONDITION));
        assert!(!rules.contains(rules::RULE_VOID_MAIN_SIGNATURE));
        assert!(!rules.contains(rules::RULE_FUNCTION_BRACE_NEWLINE));
        assert!(!rules.contains(rules::RULE_UNUSED_INCLUDE));
    }

    #[test]
    fn uses_tree_sitter_for_multiline_c_syntax() {
        let source = r#"
int main(void)
{
    int used = 0,
        unused = 1;

    while (used);

    if ((used = 3))
        used++;

    return used;
}
"#;

        let report = rules::analyze_source("ast.c", Language::C, source);
        let rules = rules(&report);
        assert!(rules.contains(rules::RULE_UNUSED_VARIABLE));
        assert!(rules.contains(rules::RULE_ASSIGNMENT_IN_CONDITION));
        assert!(rules.contains(rules::RULE_EMPTY_CONTROL_STATEMENT));
        assert!(!rules.contains(rules::RULE_FUNCTION_BRACE_NEWLINE));
    }

    #[test]
    fn detects_cpp_v1_rules() {
        let source = r#"
#include <iostream>
using namespace std;

int main()
{
    int* value = new int(1);
    int narrowed = (int)3.14;
    if (*value = 2);
    if (value == NULL) return 0;
    if(value) return narrowed;
    cout << narrowed << endl;
    delete value;
    try {
        throw 1;
    } catch (...) {
        return 2;
    }
    if (value) {
        while (value) {
            if (value) {
                if (value) {
                    if (value) {
                        return 1;
                    }
                }
            }
        }
    }
    return 0;
}
"#;

        let report = rules::analyze_source("bad.hpp", Language::Cpp, source);
        let rules = rules(&report);
        assert!(rules.contains(rules::RULE_CPP_HEADER_USING_NAMESPACE));
        assert!(rules.contains(rules::RULE_CPP_USING_NAMESPACE));
        assert!(rules.contains(rules::RULE_CPP_RAW_NEW_DELETE));
        assert!(rules.contains(rules::RULE_CPP_C_STYLE_CAST));
        assert!(rules.contains(rules::RULE_CPP_CATCH_ALL));
        assert!(rules.contains(rules::RULE_CPP_STD_ENDL));
        assert!(rules.contains(rules::RULE_CPP_NULL_LITERAL));
        assert!(rules.contains(rules::RULE_CONTROL_KEYWORD_SPACE));
        assert!(rules.contains(rules::RULE_ASSIGNMENT_IN_CONDITION));
        assert!(rules.contains(rules::RULE_DEEP_NESTING));
    }

    #[test]
    fn detects_rust_v1_rules() {
        let source = r#"
#[allow(dead_code)]
unsafe fn poke(ptr: *mut i32) {
    unsafe {
        *ptr = 1;
    }
}

impl Big {
    fn run(&self) {
        let value = Some(1).unwrap();
        let shared: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let name = String::from("tmp");
        let _copy = name.clone();
        dbg!(&shared);
        let _small = value as u8;
        let _raw: u32 = unsafe { std::mem::transmute([0u8; 4]) };
        if value > 0 {
            while value > 0 {
                if value > 0 {
                    if value > 0 {
                        if value > 0 {
                            panic!("bad");
                        }
                    }
                }
            }
        }
    }

    fn a01(&self) {}
    fn a02(&self) {}
    fn a03(&self) {}
    fn a04(&self) {}
    fn a05(&self) {}
    fn a06(&self) {}
    fn a07(&self) {}
    fn a08(&self) {}
    fn a09(&self) {}
    fn a10(&self) {}
    fn a11(&self) {}
    fn a12(&self) {}
    fn a13(&self) {}
    fn a14(&self) {}
    fn a15(&self) {}
    fn a16(&self) {}
    fn a17(&self) {}
    fn a18(&self) {}
    fn a19(&self) {}
    fn a20(&self) {}
    fn a21(&self) {}
    fn a22(&self) {}
    fn a23(&self) {}
    fn a24(&self) {}
    fn a25(&self) {}
}
"#;

        let report = rules::analyze_source("bad.rs", Language::Rust, source);
        let rules = rules(&report);
        assert!(rules.contains(rules::RULE_RUST_UNSAFE));
        assert!(rules.contains(rules::RULE_RUST_PANIC_UNWRAP));
        assert!(rules.contains(rules::RULE_RUST_ALLOW_ATTR));
        assert!(rules.contains(rules::RULE_RUST_LARGE_IMPL));
        assert!(rules.contains(rules::RULE_RUST_DBG_MACRO));
        assert!(rules.contains(rules::RULE_RUST_CLONE_NOISE));
        assert!(rules.contains(rules::RULE_RUST_COMPLEX_TYPE));
        assert!(rules.contains(rules::RULE_RUST_TRANSMUTE));
        assert!(rules.contains(rules::RULE_RUST_AS_CAST));
        assert!(rules.contains(rules::RULE_RUST_UNSAFE_WITHOUT_SAFETY));
        assert!(rules.contains(rules::RULE_DEEP_NESTING));
    }

    #[test]
    fn parses_arguments() {
        let args = vec![
            "-f".to_string(),
            "main.c".to_string(),
            "-d".to_string(),
            "src".to_string(),
            "--yaml".to_string(),
            "-o".to_string(),
            "report.md".to_string(),
        ];
        let config = parse_args(&args).unwrap();
        assert!(config.yaml);
        assert_eq!(
            config.output.as_deref(),
            Some(std::path::Path::new("report.md"))
        );
        assert_eq!(config.targets.len(), 2);
        assert!(!config.ai_review);
    }

    #[test]
    fn parses_ai_review_arguments() {
        let args = vec![
            "-f".to_string(),
            "main.c".to_string(),
            "--aireview".to_string(),
        ];
        let config = parse_args(&args).unwrap();
        assert!(config.ai_review);
    }

    #[test]
    fn rejects_removed_configuration_flags() {
        for flag in [
            "--ignore=unused-include",
            "--json",
            "-j",
            "--ai-model=x",
            "--ai-base-url=x",
        ] {
            let args = vec!["-f".to_string(), "main.c".to_string(), flag.to_string()];
            assert!(parse_args(&args).is_err(), "{flag} should not be accepted");
        }
    }

    #[test]
    fn detects_supported_languages() {
        assert_eq!(Language::from_extension("c"), Some(Language::C));
        assert_eq!(Language::from_extension("h"), Some(Language::C));
        assert_eq!(Language::from_extension("cpp"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("hpp"), Some(Language::Cpp));
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
    }

    #[test]
    fn prints_valid_yaml_shape() {
        let report =
            rules::analyze_source("ok.c", Language::C, "int main(void)\n{\n    return 0;\n}\n");
        let summary = build_summary(&report.issues, 1, report.line_count);
        let result = AnalysisResult {
            files: vec![report],
            issues: Vec::new(),
            summary,
            ai_review: None,
        };
        let yaml = build_report(&result, true, false);
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml).unwrap();
        assert!(parsed.get("files").is_some());
        assert!(parsed.get("summary").is_some());
        assert!(parsed["summary"].get("score").is_some());
    }

    #[test]
    fn text_report_contains_rich_summary() {
        let report =
            rules::analyze_source("ok.c", Language::C, "int main(void)\n{\nreturn 0;\n}\n");
        let summary = build_summary(&report.issues, 1, report.line_count);
        let text = build_report(
            &AnalysisResult {
                files: vec![report],
                issues: Vec::new(),
                summary,
                ai_review: Some(AiReviewResult {
                    provider: "deepseek".to_string(),
                    model: "deepseek-v4-pro".to_string(),
                    score: 72,
                    summary: "AI 认为整体可维护，但需要继续收敛边界风险。".to_string(),
                    recommendations: vec!["优先处理 error 级问题".to_string()],
                    raw_response: "{}".to_string(),
                }),
            },
            false,
            false,
        );

        assert!(text.contains("=== CRQA 代码质量检查 ==="));
        assert!(text.contains("=== 统计报告 ==="));
        assert!(text.contains("质量等级:"));
        assert!(text.contains("处理建议:"));
        assert!(text.contains("=== DeepSeek Review ==="));
        assert!(text.contains("评分: 72/100"));
    }

    #[test]
    fn quiet_hides_non_errors_without_changing_summary() {
        let report = rules::analyze_source(
            "bad.rs",
            Language::Rust,
            "fn main() {\n    let value = Some(1).unwrap();\n}\n",
        );
        let summary = build_summary(&report.issues, 1, report.line_count);
        let result = AnalysisResult {
            issues: report.issues.clone(),
            files: vec![report],
            summary,
            ai_review: None,
        };

        let text = build_report(&result, false, true);

        assert!(text.contains("警告: 1"));
        assert!(text.contains("代码质量评分: 94/100"));
        assert!(!text.contains("rust-panic-unwrap"));
    }

    #[test]
    fn avoids_some_text_rule_false_positives() {
        let rust = rules::analyze_source(
            "ok.rs",
            Language::Rust,
            "fn main() {\n    let panic_count = 1;\n    let unwrap_value = panic_count;\n    println!(\"{}\", unwrap_value);\n}\n",
        );
        assert!(!rules(&rust).contains(rules::RULE_RUST_PANIC_UNWRAP));

        let cpp = rules::analyze_source(
            "ok.cpp",
            Language::Cpp,
            "#include <iostream>\nint main()\n{\n    int endl = 1;\n    std::cout << endl << '\\n';\n    return 0;\n}\n",
        );
        assert!(!rules(&cpp).contains(rules::RULE_CPP_STD_ENDL));
    }
}
