use std::collections::BTreeMap;

use crate::model::{AiReviewResult, AnalysisResult, FileReport, Severity};

pub fn build_report(result: &AnalysisResult, yaml: bool, quiet: bool) -> String {
    if yaml {
        serde_yaml_ng::to_string(result).expect("analysis result should serialize to YAML")
    } else {
        build_text_report(result, quiet)
    }
}

fn build_text_report(result: &AnalysisResult, quiet: bool) -> String {
    let mut output = String::new();
    output.push_str("=== CRQA 代码质量检查 ===\n");
    output.push_str("静态质量分析\n\n");

    for report in &result.files {
        render_file_report(report, quiet, &mut output);
    }

    output.push_str("=== 统计报告 ===\n");
    output.push_str(&format!(
        "错误: {} | 警告: {} | 建议: {}\n",
        result.summary.errors, result.summary.warnings, result.summary.infos
    ));
    output.push_str(&format!(
        "检查文件: {} | 总行数: {}\n",
        result.summary.files, result.summary.total_lines
    ));
    output.push_str(&format!("代码质量评分: {}/100\n", result.summary.score));
    output.push_str(&format!(
        "质量等级: {}\n",
        quality_label(result.summary.score)
    ));
    render_hot_rules(result, quiet, &mut output);
    output.push_str(&format!(
        "处理建议: {}\n",
        action_hint(result.summary.errors, result.summary.warnings)
    ));
    if let Some(ai_review) = &result.ai_review {
        render_ai_review(ai_review, &mut output);
    }
    output
}

fn render_file_report(report: &FileReport, quiet: bool, output: &mut String) {
    let shown_count = visible_issue_count(&report.issues, quiet);
    output.push_str(&format!(
        "--- {} 代码检查 ---\n",
        report.language.display_name()
    ));
    output.push_str(&format!("文件: {}\n", report.path));
    output.push_str(&format!("行数: {}\n", report.line_count));
    if quiet {
        output.push_str(&format!("展示问题: {shown_count}\n"));
    } else {
        output.push_str(&format!("问题: {shown_count}\n"));
    }

    if shown_count == 0 {
        if quiet && !report.issues.is_empty() {
            output.push_str("状态: quiet 模式已隐藏非 error 级问题。\n\n");
        } else {
            output.push_str("状态: 未发现问题。\n\n");
        }
        return;
    }

    output.push('\n');
    for issue in report
        .issues
        .iter()
        .filter(|issue| is_visible_issue(issue, quiet))
    {
        output.push_str(&format!(
            "[{}] {} | line {:>3} | {}\n",
            issue.severity, issue.language, issue.line, issue.rule
        ));
        output.push_str(&format!("  -> {}\n", issue.message));
    }

    let (errors, warnings, infos) = issue_counts(&report.issues, quiet);
    output.push_str(&format!(
        "\n小结: error {} | warning {} | info {}\n",
        errors, warnings, infos
    ));
    output.push('\n');
}

fn render_hot_rules(result: &AnalysisResult, quiet: bool, output: &mut String) {
    if !result
        .issues
        .iter()
        .any(|issue| is_visible_issue(issue, quiet))
    {
        return;
    }

    let mut counts = BTreeMap::new();
    for issue in result
        .issues
        .iter()
        .filter(|issue| is_visible_issue(issue, quiet))
    {
        *counts.entry(issue.rule.as_str()).or_insert(0usize) += 1;
    }

    let mut hot_rules = counts.into_iter().collect::<Vec<_>>();
    hot_rules.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));

    let summary = hot_rules
        .into_iter()
        .take(3)
        .map(|(rule, count)| format!("{rule} x{count}"))
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!("规则热点: {summary}\n"));
}

fn render_ai_review(ai_review: &AiReviewResult, output: &mut String) {
    output.push_str("\n=== DeepSeek Review ===\n");
    output.push_str(&format!(
        "服务商: {} | 模型: {}\n",
        ai_review.provider, ai_review.model
    ));
    output.push_str(&format!("评分: {}/100\n", ai_review.score));
    output.push_str(&format!("摘要: {}\n", ai_review.summary));
    if !ai_review.recommendations.is_empty() {
        output.push_str("建议:\n");
        for (index, recommendation) in ai_review.recommendations.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", index + 1, recommendation));
        }
    }
}

fn is_visible_issue(issue: &crate::Issue, quiet: bool) -> bool {
    !quiet || issue.severity == Severity::Error
}

fn visible_issue_count(issues: &[crate::Issue], quiet: bool) -> usize {
    issues
        .iter()
        .filter(|issue| is_visible_issue(issue, quiet))
        .count()
}

fn issue_counts(issues: &[crate::Issue], quiet: bool) -> (usize, usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;
    let mut infos = 0;
    for issue in issues.iter().filter(|issue| is_visible_issue(issue, quiet)) {
        match issue.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => infos += 1,
        }
    }
    (errors, warnings, infos)
}

fn quality_label(score: u8) -> &'static str {
    match score {
        90..=100 => "优秀",
        75..=89 => "良好",
        60..=74 => "可维护",
        40..=59 => "需要治理",
        20..=39 => "风险较高",
        _ => "需要立即整理",
    }
}

fn action_hint(errors: usize, warnings: usize) -> &'static str {
    if errors > 0 {
        "优先处理 error 级问题。"
    } else if warnings > 0 {
        "复查 warning，重点关注资源管理、unsafe 和裸 new/delete。"
    } else {
        "当前未发现需要处理的问题。"
    }
}
