use std::env;
use std::fs;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::model::{AiReviewResult, AnalysisResult, FileReport, Severity};

const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
const AI_PROVIDER: &str = "deepseek";
const AI_MODEL: &str = "deepseek-v4-pro";
const CHAT_COMPLETIONS_URL: &str = "https://api.deepseek.com/chat/completions";
const MAX_FILES_FOR_AI: usize = 3;
const MAX_SOURCE_CHARS_PER_FILE: usize = 5000;
const MAX_PROMPT_CHARS: usize = 18000;

pub fn run_ai_review(result: &AnalysisResult) -> Result<AiReviewResult, String> {
    let api_key = env::var(API_KEY_ENV)
        .map_err(|_| format!("--aireview requires {API_KEY_ENV} environment variable"))?;
    let prompt = build_review_prompt(result);
    let raw_response = call_deepseek(&api_key, &prompt)?;
    let parsed = parse_ai_review_content(&raw_response)?;

    Ok(AiReviewResult {
        provider: AI_PROVIDER.to_string(),
        model: AI_MODEL.to_string(),
        score: parsed.score.clamp(0.0, 100.0).round() as u8,
        summary: parsed.summary,
        recommendations: parsed.recommendations,
        raw_response,
    })
}

fn call_deepseek(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|err| format!("failed to build AI HTTP client: {err}"))?;

    let response = client
        .post(CHAT_COMPLETIONS_URL)
        .bearer_auth(api_key)
        .json(&json!({
            "model": AI_MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": "你是 CRQA 的 AI 代码质量评分器。只基于用户提供的静态分析结果和源码节选评分。输出必须是 JSON，不要输出 Markdown。"
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "stream": false,
            "max_tokens": 1200,
            "thinking": { "type": "disabled" },
            "response_format": { "type": "json_object" }
        }))
        .send()
        .map_err(|err| format!("AI request failed: {err}"))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|err| format!("failed to read AI response: {err}"))?;

    if !status.is_success() {
        return Err(format!(
            "AI request returned HTTP {status}: {}",
            truncate_for_error(&body)
        ));
    }

    let response: ChatCompletionResponse = serde_json::from_str(&body)
        .map_err(|err| format!("failed to parse AI response envelope: {err}"))?;
    response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| "AI response did not contain message content".to_string())
}

fn build_review_prompt(result: &AnalysisResult) -> String {
    let mut prompt = String::new();
    prompt.push_str(
        "请对这个 C/C++/Rust 项目的代码质量做 AI 二次评分。\n\
         要求：\n\
         1. score 为 0-100，越高越好。\n\
         2. summary 用中文，简短但具体。\n\
         3. recommendations 给 3-6 条中文建议，优先级从高到低。\n\
         4. 只输出 JSON：{\"score\": number, \"summary\": string, \"recommendations\": string[]}。\n\n",
    );
    prompt.push_str(&format!(
        "本地 CRQA 汇总：error={} warning={} info={} files={} lines={} local_score={}/100\n\n",
        result.summary.errors,
        result.summary.warnings,
        result.summary.infos,
        result.summary.files,
        result.summary.total_lines,
        result.summary.score
    ));

    for report in worst_reports(result) {
        prompt.push_str(&format!(
            "## 文件: {} ({}, {} lines)\n",
            report.path, report.language, report.line_count
        ));
        prompt.push_str("问题列表：\n");
        if report.issues.is_empty() {
            prompt.push_str("- 无本地规则问题\n");
        } else {
            for issue in &report.issues {
                prompt.push_str(&format!(
                    "- [{}] line {} {}: {}\n",
                    issue.severity, issue.line, issue.rule, issue.message
                ));
            }
        }
        prompt.push_str("源码节选：\n```text\n");
        prompt.push_str(&read_source_excerpt(&report.path));
        prompt.push_str("\n```\n\n");

        if prompt.len() > MAX_PROMPT_CHARS {
            prompt = prompt.chars().take(MAX_PROMPT_CHARS).collect();
            prompt.push_str("\n\n[内容因长度限制被截断]\n");
            break;
        }
    }

    prompt
}

fn worst_reports(result: &AnalysisResult) -> Vec<&FileReport> {
    let mut reports = result.files.iter().collect::<Vec<_>>();
    reports.sort_by(|left, right| {
        file_penalty(right)
            .cmp(&file_penalty(left))
            .then_with(|| left.path.cmp(&right.path))
    });
    reports.into_iter().take(MAX_FILES_FOR_AI).collect()
}

fn file_penalty(report: &FileReport) -> usize {
    report
        .issues
        .iter()
        .map(|issue| match issue.severity {
            Severity::Error => 12,
            Severity::Warning => 5,
            Severity::Info => 2,
        })
        .sum()
}

fn read_source_excerpt(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(source) if source.len() > MAX_SOURCE_CHARS_PER_FILE => {
            let mut excerpt = source
                .chars()
                .take(MAX_SOURCE_CHARS_PER_FILE)
                .collect::<String>();
            excerpt.push_str("\n[源码节选因长度限制被截断]");
            excerpt
        }
        Ok(source) => source,
        Err(err) => format!("[无法读取源码: {err}]"),
    }
}

fn parse_ai_review_content(content: &str) -> Result<ParsedAiReview, String> {
    let json_text = strip_json_fence(content.trim());
    serde_json::from_str(json_text).map_err(|err| format!("failed to parse AI review JSON: {err}"))
}

fn strip_json_fence(content: &str) -> &str {
    let content = content.trim();
    if let Some(rest) = content.strip_prefix("```json") {
        return rest.trim().trim_end_matches("```").trim();
    }
    if let Some(rest) = content.strip_prefix("```") {
        return rest.trim().trim_end_matches("```").trim();
    }
    content
}

fn truncate_for_error(body: &str) -> String {
    const MAX_ERROR_BODY_CHARS: usize = 300;
    let mut text = body.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    if body.chars().count() > MAX_ERROR_BODY_CHARS {
        text.push_str("...");
    }
    text
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ParsedAiReview {
    score: f64,
    summary: String,
    recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_ai_review_json() {
        let parsed = parse_ai_review_content(
            r#"{"score":88,"summary":"整体不错","recommendations":["减少 unsafe"]}"#,
        )
        .unwrap();
        assert_eq!(parsed.score, 88.0);
        assert_eq!(parsed.recommendations.len(), 1);
    }

    #[test]
    fn parses_fenced_ai_review_json() {
        let parsed = parse_ai_review_content(
            r#"```json
{"score":66,"summary":"有风险","recommendations":["先修 error","补充测试"]}
```"#,
        )
        .unwrap();
        assert_eq!(parsed.score, 66.0);
        assert_eq!(parsed.recommendations[0], "先修 error");
    }

    #[test]
    fn uses_fixed_deepseek_defaults() {
        assert_eq!(AI_PROVIDER, "deepseek");
        assert_eq!(AI_MODEL, "deepseek-v4-pro");
        assert_eq!(
            CHAT_COMPLETIONS_URL,
            "https://api.deepseek.com/chat/completions"
        );
    }
}
