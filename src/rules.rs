use std::path::Path;

use tree_sitter::Node;

use crate::model::{FileReport, Language, Severity, issue};
use crate::parser::parse_tree;

pub const RULE_FUNCTION_BRACE_NEWLINE: &str = "function-brace-newline";
pub const RULE_SINGLE_LINE_FOR_BRACES: &str = "single-line-for-braces";
pub const RULE_VOID_MAIN_SIGNATURE: &str = "void-main-signature";
pub const RULE_UNUSED_INCLUDE: &str = "unused-include";
pub const RULE_UNUSED_VARIABLE: &str = "unused-variable";
pub const RULE_ASSIGNMENT_IN_CONDITION: &str = "assignment-in-condition";
pub const RULE_EQUALITY_AS_STATEMENT: &str = "equality-as-statement";
pub const RULE_EMPTY_CONTROL_STATEMENT: &str = "empty-control-statement";
pub const RULE_LONG_FUNCTION: &str = "long-function";
pub const RULE_DEEP_NESTING: &str = "deep-nesting";
pub const RULE_CONTROL_KEYWORD_SPACE: &str = "control-keyword-space";
pub const RULE_C_DANGEROUS_API: &str = "c-dangerous-api";
pub const RULE_C_MACRO_SAFETY: &str = "c-macro-safety";
pub const RULE_C_HEADER_GUARD: &str = "c-header-guard";
pub const RULE_C_UNCHECKED_RESOURCE: &str = "c-unchecked-resource";
pub const RULE_C_GOTO_USAGE: &str = "c-goto-usage";
pub const RULE_C_ALLOC_WITHOUT_FREE: &str = "c-alloc-without-free";
pub const RULE_CPP_TOO_MANY_INCLUDES: &str = "cpp-too-many-includes";
pub const RULE_CPP_USING_NAMESPACE: &str = "cpp-using-namespace";
pub const RULE_CPP_HEADER_USING_NAMESPACE: &str = "cpp-header-using-namespace";
pub const RULE_CPP_RAW_NEW_DELETE: &str = "cpp-raw-new-delete";
pub const RULE_CPP_C_STYLE_CAST: &str = "cpp-c-style-cast";
pub const RULE_CPP_CATCH_ALL: &str = "cpp-catch-all";
pub const RULE_CPP_STD_ENDL: &str = "cpp-std-endl";
pub const RULE_CPP_NULL_LITERAL: &str = "cpp-null-literal";
pub const RULE_RUST_PANIC_UNWRAP: &str = "rust-panic-unwrap";
pub const RULE_RUST_UNSAFE: &str = "rust-unsafe";
pub const RULE_RUST_LARGE_IMPL: &str = "rust-large-impl";
pub const RULE_RUST_ALLOW_ATTR: &str = "rust-allow-attr";
pub const RULE_RUST_DBG_MACRO: &str = "rust-dbg-macro";
pub const RULE_RUST_CLONE_NOISE: &str = "rust-clone-noise";
pub const RULE_RUST_COMPLEX_TYPE: &str = "rust-complex-type";
pub const RULE_RUST_TRANSMUTE: &str = "rust-transmute";
pub const RULE_RUST_AS_CAST: &str = "rust-as-cast";
pub const RULE_RUST_UNSAFE_WITHOUT_SAFETY: &str = "rust-unsafe-without-safety-comment";

#[derive(Debug, Clone)]
struct IncludeDecl {
    line: usize,
    header: String,
}

#[derive(Debug, Clone)]
struct VarDecl {
    line: usize,
    name: String,
}

#[derive(Debug, Clone)]
struct FunctionState {
    name: String,
    start_line: usize,
}

pub fn analyze_source(file: &str, language: Language, source: &str) -> FileReport {
    let original_lines: Vec<&str> = source.lines().collect();
    let sanitized = sanitize_source(source);
    let sanitized_lines: Vec<&str> = sanitized.lines().collect();
    let line_count = original_lines.len();

    let mut issues = Vec::new();
    if let Some(tree) = parse_tree(language, source) {
        let root = tree.root_node();
        if root.has_error() {
            issues.extend(check_text_fallback(file, language, &sanitized_lines));
        } else {
            match language {
                Language::C => {
                    issues.extend(check_c_ast_rules(file, language, source, root));
                    issues.extend(check_keyword_spacing_rules(
                        file,
                        language,
                        &sanitized_lines,
                    ));
                    issues.extend(check_c_text_rules(
                        file,
                        language,
                        &sanitized_lines,
                        &original_lines,
                    ));
                    issues.extend(check_unused_includes(file, language, &sanitized_lines));
                    issues.extend(check_unused_variables_ast(file, language, source, root));
                }
                Language::Cpp => {
                    issues.extend(check_cpp_ast_rules(file, language, source, root));
                    issues.extend(check_keyword_spacing_rules(
                        file,
                        language,
                        &sanitized_lines,
                    ));
                    issues.extend(check_cpp_text_rules(
                        file,
                        language,
                        &sanitized_lines,
                        &original_lines,
                    ));
                }
                Language::Rust => {
                    issues.extend(check_rust_ast_rules(file, language, root));
                    issues.extend(check_rust_text_rules(
                        file,
                        language,
                        &sanitized_lines,
                        &original_lines,
                    ));
                }
            }
        }
    } else {
        issues.extend(check_text_fallback(file, language, &sanitized_lines));
    }

    issues.sort_by(crate::model::compare_issues);
    issues.dedup();

    FileReport {
        path: file.to_string(),
        language,
        line_count,
        issues,
    }
}

fn check_text_fallback(
    file: &str,
    language: Language,
    sanitized_lines: &[&str],
) -> Vec<crate::Issue> {
    match language {
        Language::C => {
            let mut issues = Vec::new();
            issues.extend(check_c_line_rules(file, language, sanitized_lines));
            issues.extend(check_c_text_rules(
                file,
                language,
                sanitized_lines,
                sanitized_lines,
            ));
            issues.extend(check_unused_includes(file, language, sanitized_lines));
            issues.extend(check_unused_variables(file, language, sanitized_lines));
            issues.extend(check_function_metrics(file, language, sanitized_lines));
            issues
        }
        Language::Cpp => {
            let mut issues = Vec::new();
            issues.extend(check_cpp_line_rules(file, language, sanitized_lines));
            issues.extend(check_function_metrics(file, language, sanitized_lines));
            issues.extend(check_cpp_text_rules(
                file,
                language,
                sanitized_lines,
                sanitized_lines,
            ));
            issues
        }
        Language::Rust => check_rust_text_rules(file, language, sanitized_lines, sanitized_lines),
    }
}

fn check_c_ast_rules(
    file: &str,
    language: Language,
    source: &str,
    root: Node<'_>,
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();

    visit_named_nodes(root, &mut |node| match node.kind() {
        "function_definition" => check_c_ast_function(file, language, source, node, &mut issues),
        "for_statement" => check_ast_for_statement(file, language, source, node, &mut issues),
        "if_statement" => check_ast_if_statement(file, language, source, node, &mut issues),
        "while_statement" => check_ast_while_statement(file, language, source, node, &mut issues),
        "expression_statement" => {
            check_ast_expression_statement(file, language, source, node, &mut issues);
        }
        _ => {}
    });

    issues
}

fn check_cpp_ast_rules(
    file: &str,
    language: Language,
    source: &str,
    root: Node<'_>,
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();

    visit_named_nodes(root, &mut |node| match node.kind() {
        "function_definition" => check_generic_function(file, language, source, node, &mut issues),
        "for_statement" => check_ast_for_statement(file, language, source, node, &mut issues),
        "if_statement" => check_ast_if_statement(file, language, source, node, &mut issues),
        "while_statement" => check_ast_while_statement(file, language, source, node, &mut issues),
        _ => {}
    });

    issues
}

fn check_rust_ast_rules(file: &str, language: Language, root: Node<'_>) -> Vec<crate::Issue> {
    let mut issues = Vec::new();

    visit_named_nodes(root, &mut |node| match node.kind() {
        "function_item" => check_rust_function(file, language, node, &mut issues),
        "impl_item" => check_rust_impl(file, language, node, &mut issues),
        _ => {}
    });

    if let Some(deep_node) = first_deep_control_node_for_language(root, language, 0) {
        issues.push(issue(
            file,
            language,
            deep_node.start_position().row + 1,
            Severity::Warning,
            RULE_DEEP_NESTING,
            "嵌套深度超过 4 层，建议重构",
        ));
    }

    issues
}

fn check_c_ast_function(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    check_generic_function(file, language, source, node, issues);

    if let Some(body) = node.child_by_field_name("body")
        && node.start_position().row == body.start_position().row
    {
        issues.push(issue(
            file,
            language,
            body.start_position().row + 1,
            Severity::Error,
            RULE_FUNCTION_BRACE_NEWLINE,
            "函数定义的首个左花括号应另起一行",
        ));
    }

    let Some(type_node) = node.child_by_field_name("type") else {
        return;
    };
    let Some(declarator) = node.child_by_field_name("declarator") else {
        return;
    };
    let Some(name) = find_declarator_identifier(declarator, source, true) else {
        return;
    };
    let Some(parameters) = find_first_descendant_kind(declarator, "parameter_list") else {
        return;
    };

    if name == "main"
        && node_text(type_node, source).trim() == "void"
        && parameter_list_is_empty(parameters, source)
    {
        issues.push(issue(
            file,
            language,
            node.start_position().row + 1,
            Severity::Error,
            RULE_VOID_MAIN_SIGNATURE,
            "建议将 void main() 改为 int main(void)",
        ));
    }
}

fn check_generic_function(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    if node_line_count(node) > 100 {
        let name = node
            .child_by_field_name("declarator")
            .and_then(|declarator| find_declarator_identifier(declarator, source, true))
            .unwrap_or_else(|| "<anonymous>".to_string());
        issues.push(issue(
            file,
            language,
            node.start_position().row + 1,
            Severity::Info,
            RULE_LONG_FUNCTION,
            format!(
                "函数 {}() 超过 100 行，当前 {} 行，建议拆分",
                name,
                node_line_count(node)
            ),
        ));
    }

    if let Some(body) = node.child_by_field_name("body")
        && let Some(deep_node) = first_deep_control_node_for_language(body, language, 0)
    {
        issues.push(issue(
            file,
            language,
            deep_node.start_position().row + 1,
            Severity::Warning,
            RULE_DEEP_NESTING,
            "嵌套深度超过 4 层，建议重构",
        ));
    }
}

fn check_rust_function(
    file: &str,
    language: Language,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    if node_line_count(node) > 100 {
        issues.push(issue(
            file,
            language,
            node.start_position().row + 1,
            Severity::Info,
            RULE_LONG_FUNCTION,
            format!(
                "Rust 函数超过 100 行，当前 {} 行，建议拆分",
                node_line_count(node)
            ),
        ));
    }
}

fn check_rust_impl(file: &str, language: Language, node: Node<'_>, issues: &mut Vec<crate::Issue>) {
    if node_line_count(node) > 40 {
        issues.push(issue(
            file,
            language,
            node.start_position().row + 1,
            Severity::Info,
            RULE_RUST_LARGE_IMPL,
            format!(
                "impl 块超过 40 行，当前 {} 行，建议拆分职责",
                node_line_count(node)
            ),
        ));
    }
}

fn check_ast_for_statement(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    push_condition_assignment_issue(
        file,
        language,
        source,
        node,
        "for 条件段中疑似误用赋值运算符 =，请确认是否应为 ==",
        issues,
    );

    if let Some(body) = node.child_by_field_name("body") {
        if is_empty_expression_statement(body) {
            issues.push(issue(
                file,
                language,
                body.start_position().row + 1,
                Severity::Error,
                RULE_EMPTY_CONTROL_STATEMENT,
                "for 语句后不应出现空语句分号",
            ));
        } else if language == Language::C
            && body.kind() == "compound_statement"
            && body.start_position().row == body.end_position().row
        {
            issues.push(issue(
                file,
                language,
                node.start_position().row + 1,
                Severity::Warning,
                RULE_SINGLE_LINE_FOR_BRACES,
                "单行 for 循环不应使用花括号",
            ));
        }
    }
}

fn check_ast_if_statement(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    push_condition_assignment_issue(
        file,
        language,
        source,
        node,
        CONDITION_ASSIGNMENT_MESSAGE,
        issues,
    );
    push_empty_statement_issue(file, language, node, "consequence", "if", issues);
}

fn check_ast_while_statement(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    push_condition_assignment_issue(
        file,
        language,
        source,
        node,
        CONDITION_ASSIGNMENT_MESSAGE,
        issues,
    );
    push_empty_statement_issue(file, language, node, "body", "while", issues);
}

fn check_ast_expression_statement(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    issues: &mut Vec<crate::Issue>,
) {
    let Some(expression) = first_named_child(node) else {
        return;
    };

    if expression.kind() == "binary_expression" && binary_operator_is(expression, source, "==") {
        issues.push(issue(
            file,
            language,
            node.start_position().row + 1,
            Severity::Error,
            RULE_EQUALITY_AS_STATEMENT,
            "独立语句中出现 ==，疑似将赋值 = 误写为比较 ==",
        ));
    }
}

fn check_unused_variables_ast(
    file: &str,
    language: Language,
    source: &str,
    root: Node<'_>,
) -> Vec<crate::Issue> {
    let mut declarations = Vec::new();
    visit_named_nodes(root, &mut |node| {
        if node.kind() != "declaration" {
            return;
        }

        let mut cursor = node.walk();
        for declarator in node.children_by_field_name("declarator", &mut cursor) {
            let Some(name) = find_declarator_identifier(declarator, source, false) else {
                continue;
            };
            declarations.push(VarDecl {
                line: declarator.start_position().row + 1,
                name,
            });
        }
    });

    let mut issues = Vec::new();
    for declaration in declarations {
        if declaration.name.starts_with('_') {
            continue;
        }
        if count_identifier_nodes(root, source, &declaration.name) <= 1 {
            issues.push(issue(
                file,
                language,
                declaration.line,
                Severity::Warning,
                RULE_UNUSED_VARIABLE,
                format!("变量 '{}' 定义后未使用", declaration.name),
            ));
        }
    }
    issues
}

const CONDITION_ASSIGNMENT_MESSAGE: &str = "条件表达式中疑似误用赋值运算符 =，请确认是否应为 ==";

fn push_condition_assignment_issue(
    file: &str,
    language: Language,
    source: &str,
    node: Node<'_>,
    message: &str,
    issues: &mut Vec<crate::Issue>,
) {
    let Some(condition) = node.child_by_field_name("condition") else {
        return;
    };
    if !contains_plain_assignment(condition, source) {
        return;
    }

    issues.push(issue(
        file,
        language,
        condition.start_position().row + 1,
        Severity::Error,
        RULE_ASSIGNMENT_IN_CONDITION,
        message,
    ));
}

fn push_empty_statement_issue(
    file: &str,
    language: Language,
    node: Node<'_>,
    field_name: &str,
    keyword: &str,
    issues: &mut Vec<crate::Issue>,
) {
    let Some(body) = node.child_by_field_name(field_name) else {
        return;
    };
    if !is_empty_expression_statement(body) {
        return;
    }

    issues.push(issue(
        file,
        language,
        body.start_position().row + 1,
        Severity::Error,
        RULE_EMPTY_CONTROL_STATEMENT,
        format!("{keyword} 语句后不应出现空语句分号"),
    ));
}

fn visit_named_nodes(node: Node<'_>, visitor: &mut impl FnMut(Node<'_>)) {
    visitor(node);
    for index in 0..node.named_child_count() {
        if let Some(child) = node.named_child(index as u32) {
            visit_named_nodes(child, visitor);
        }
    }
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}

fn node_line_count(node: Node<'_>) -> usize {
    node.end_position()
        .row
        .saturating_sub(node.start_position().row)
        + 1
}

fn parameter_list_is_empty(node: Node<'_>, source: &str) -> bool {
    let text = node_text(node, source).trim();
    text.starts_with('(')
        && text.ends_with(')')
        && text[1..text.len().saturating_sub(1)].trim().is_empty()
}

fn find_first_descendant_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }

    for index in 0..node.named_child_count() {
        if let Some(child) = node.named_child(index as u32)
            && let Some(found) = find_first_descendant_kind(child, kind)
        {
            return Some(found);
        }
    }
    None
}

fn find_declarator_identifier(
    node: Node<'_>,
    source: &str,
    allow_function_declarator: bool,
) -> Option<String> {
    match node.kind() {
        "identifier" => Some(node_text(node, source).to_string()),
        "function_declarator" if !allow_function_declarator => None,
        "function_declarator"
        | "init_declarator"
        | "pointer_declarator"
        | "array_declarator"
        | "qualified_identifier" => node
            .child_by_field_name("declarator")
            .and_then(|child| find_declarator_identifier(child, source, allow_function_declarator))
            .or_else(|| {
                node.child_by_field_name("name")
                    .map(|child| node_text(child, source).to_string())
            }),
        "parenthesized_declarator" => {
            for index in 0..node.named_child_count() {
                if let Some(child) = node.named_child(index as u32)
                    && let Some(identifier) =
                        find_declarator_identifier(child, source, allow_function_declarator)
                {
                    return Some(identifier);
                }
            }
            None
        }
        _ => {
            for index in 0..node.named_child_count() {
                if let Some(child) = node.named_child(index as u32)
                    && let Some(identifier) =
                        find_declarator_identifier(child, source, allow_function_declarator)
                {
                    return Some(identifier);
                }
            }
            None
        }
    }
}

fn contains_plain_assignment(node: Node<'_>, source: &str) -> bool {
    let mut found = false;
    visit_named_nodes(node, &mut |candidate| {
        if candidate.kind() == "assignment_expression"
            && operator_text(candidate, source).is_some_and(|operator| operator == "=")
        {
            found = true;
        }
    });
    found
}

fn binary_operator_is(node: Node<'_>, source: &str, expected: &str) -> bool {
    operator_text(node, source).is_some_and(|operator| operator == expected)
}

fn operator_text<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    node.child_by_field_name("operator")
        .map(|operator| node_text(operator, source).trim())
        .filter(|operator| !operator.is_empty())
}

fn first_named_child(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).next()
}

fn is_empty_expression_statement(node: Node<'_>) -> bool {
    node.kind() == "expression_statement" && node.named_child_count() == 0
}

fn first_deep_control_node_for_language(
    node: Node<'_>,
    language: Language,
    depth: usize,
) -> Option<Node<'_>> {
    let next_depth = if is_control_statement(language, node.kind()) {
        depth + 1
    } else {
        depth
    };
    if next_depth > 4 {
        return Some(node);
    }

    for index in 0..node.named_child_count() {
        if let Some(child) = node.named_child(index as u32)
            && let Some(found) = first_deep_control_node_for_language(child, language, next_depth)
        {
            return Some(found);
        }
    }
    None
}

fn is_control_statement(language: Language, kind: &str) -> bool {
    match language {
        Language::C | Language::Cpp => matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "do_statement"
                | "switch_statement"
        ),
        Language::Rust => matches!(
            kind,
            "if_expression"
                | "for_expression"
                | "while_expression"
                | "loop_expression"
                | "match_expression"
        ),
    }
}

fn count_identifier_nodes(root: Node<'_>, source: &str, name: &str) -> usize {
    let mut count = 0;
    visit_named_nodes(root, &mut |node| {
        if node.kind() == "identifier" && node_text(node, source) == name {
            count += 1;
        }
    });
    count
}

fn check_c_line_rules(file: &str, language: Language, lines: &[&str]) -> Vec<crate::Issue> {
    let mut issues = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();

        if violates_function_brace_newline(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Error,
                RULE_FUNCTION_BRACE_NEWLINE,
                "函数定义的首个左花括号应另起一行",
            ));
        }

        if single_line_for_has_braces(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_SINGLE_LINE_FOR_BRACES,
                "单行 for 循环不应使用花括号",
            ));
        }

        if contains_void_main_empty_params(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Error,
                RULE_VOID_MAIN_SIGNATURE,
                "建议将 void main() 改为 int main(void)",
            ));
        }

        push_keyword_spacing_issue(file, language, line_no, line, &mut issues);
        push_text_control_issues(file, language, line_no, line, &mut issues);

        if equality_used_as_statement(trimmed) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Error,
                RULE_EQUALITY_AS_STATEMENT,
                "独立语句中出现 ==，疑似将赋值 = 误写为比较 ==",
            ));
        }
    }

    issues
}

fn check_cpp_line_rules(file: &str, language: Language, lines: &[&str]) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;
        push_keyword_spacing_issue(file, language, line_no, line, &mut issues);
        push_text_control_issues(file, language, line_no, line, &mut issues);
    }
    issues
}

fn check_keyword_spacing_rules(
    file: &str,
    language: Language,
    lines: &[&str],
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        push_keyword_spacing_issue(file, language, index + 1, line, &mut issues);
    }
    issues
}

fn push_keyword_spacing_issue(
    file: &str,
    language: Language,
    line_no: usize,
    line: &str,
    issues: &mut Vec<crate::Issue>,
) {
    for keyword in ["if", "for", "while", "switch"] {
        if missing_space_after_keyword(line, keyword) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Error,
                RULE_CONTROL_KEYWORD_SPACE,
                format!("{keyword} 语句后缺少空格，建议写成 `{keyword} (...)`"),
            ));
            return;
        }
    }
}

fn check_c_text_rules(
    file: &str,
    language: Language,
    sanitized_lines: &[&str],
    original_lines: &[&str],
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    if is_c_header(file) && !has_header_guard(original_lines) {
        issues.push(issue(
            file,
            language,
            1,
            Severity::Warning,
            RULE_C_HEADER_GUARD,
            "头文件缺少 #pragma once 或传统 include guard，容易被重复包含",
        ));
    }

    for (index, line) in sanitized_lines.iter().enumerate() {
        let line_no = index + 1;

        if let Some(api) = dangerous_c_api(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_C_DANGEROUS_API,
                format!("检测到危险 C API `{api}`，建议使用带长度限制或可检查错误的替代方案"),
            ));
        }

        if resource_call_without_assignment(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_C_UNCHECKED_RESOURCE,
                "疑似直接调用资源/内存 API 且未保存返回值，建议检查失败路径和释放路径",
            ));
        }

        if looks_like_risky_macro(line) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_C_MACRO_SAFETY,
                "函数式宏包含语句块但未使用 do { ... } while (0) 包裹，调用方容易踩坑",
            ));
        }

        if contains_word(line, "goto") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_C_GOTO_USAGE,
                "检测到 goto，建议确认控制流是否可以用早返回、循环状态或小函数表达",
            ));
        }
    }

    if let Some(line_no) = first_allocation_without_free(sanitized_lines) {
        issues.push(issue(
            file,
            language,
            line_no,
            Severity::Warning,
            RULE_C_ALLOC_WITHOUT_FREE,
            "文件中检测到 malloc/calloc/realloc 但未看到 free，建议检查所有权和释放路径",
        ));
    }
    issues
}

fn is_c_header(file: &str) -> bool {
    Path::new(file)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("h"))
}

fn has_header_guard(lines: &[&str]) -> bool {
    let mut previous_ifndef = false;
    for line in lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("/*"))
        .take(8)
    {
        if line.starts_with("#pragma once") {
            return true;
        }
        if previous_ifndef && line.starts_with("#define") {
            return true;
        }
        previous_ifndef = line.starts_with("#ifndef");
    }
    false
}

fn dangerous_c_api(line: &str) -> Option<&'static str> {
    [
        "gets", "strcpy", "strcat", "sprintf", "vsprintf", "scanf", "sscanf",
    ]
    .into_iter()
    .find(|api| contains_word(line, api))
}

fn resource_call_without_assignment(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("return ") || trimmed.contains('=') {
        return false;
    }

    ["malloc", "calloc", "realloc", "fopen", "open", "socket"]
        .iter()
        .any(|api| contains_word(trimmed, api))
}

fn first_allocation_without_free(lines: &[&str]) -> Option<usize> {
    let first_alloc_line = lines
        .iter()
        .position(|line| {
            contains_word(line, "malloc")
                || contains_word(line, "calloc")
                || contains_word(line, "realloc")
        })
        .map(|index| index + 1);
    let has_free = lines.iter().any(|line| contains_word(line, "free"));

    first_alloc_line.filter(|_| !has_free)
}

fn looks_like_risky_macro(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#define")
        && trimmed.contains('(')
        && trimmed.contains('{')
        && !trimmed.contains("do")
        && !trimmed.contains("while")
}

fn push_text_control_issues(
    file: &str,
    language: Language,
    line_no: usize,
    line: &str,
    issues: &mut Vec<crate::Issue>,
) {
    for keyword in ["if", "for", "while"] {
        if control_statement_has_empty_body(line, keyword) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Error,
                RULE_EMPTY_CONTROL_STATEMENT,
                format!("{keyword} 语句后不应出现空语句分号"),
            ));
            break;
        }
    }

    if condition_has_assignment(line, "if") || condition_has_assignment(line, "while") {
        issues.push(issue(
            file,
            language,
            line_no,
            Severity::Error,
            RULE_ASSIGNMENT_IN_CONDITION,
            "条件表达式中疑似误用赋值运算符 =，请确认是否应为 ==",
        ));
    } else if for_condition_has_assignment(line) {
        issues.push(issue(
            file,
            language,
            line_no,
            Severity::Error,
            RULE_ASSIGNMENT_IN_CONDITION,
            "for 条件段中疑似误用赋值运算符 =，请确认是否应为 ==",
        ));
    }
}

fn check_cpp_text_rules(
    file: &str,
    language: Language,
    sanitized_lines: &[&str],
    original_lines: &[&str],
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    let includes = collect_includes(sanitized_lines);
    let declared_names = sanitized_lines
        .iter()
        .flat_map(|line| extract_variable_declarations(line))
        .collect::<Vec<_>>();
    if includes.len() > 20 {
        issues.push(issue(
            file,
            language,
            includes[20].line,
            Severity::Warning,
            RULE_CPP_TOO_MANY_INCLUDES,
            format!("include 数量为 {}，建议收敛头文件依赖", includes.len()),
        ));
    }

    if is_cpp_header(file) {
        for (index, line) in sanitized_lines.iter().enumerate() {
            if line.trim_start().starts_with("using namespace") {
                issues.push(issue(
                    file,
                    language,
                    index + 1,
                    Severity::Warning,
                    RULE_CPP_HEADER_USING_NAMESPACE,
                    "头文件中不应使用 using namespace，容易污染包含方命名空间",
                ));
            }
        }
    }

    for index in 0..original_lines.len() {
        let sanitized = sanitized_lines.get(index).copied().unwrap_or("");
        let line_no = index + 1;
        let trimmed = sanitized.trim();

        if trimmed.starts_with("using namespace") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_CPP_USING_NAMESPACE,
                "检测到 using namespace，建议缩小作用域或使用显式命名空间",
            ));
        }

        if contains_word(sanitized, "new") || contains_word(sanitized, "delete") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_CPP_RAW_NEW_DELETE,
                "检测到裸 new/delete，建议优先使用 RAII、智能指针或容器",
            ));
        }

        if looks_like_c_style_cast(trimmed) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_CPP_C_STYLE_CAST,
                "检测到疑似 C 风格强转，建议使用 static_cast/reinterpret_cast/const_cast 表达意图",
            ));
        }

        if trimmed.contains("catch (...)") || trimmed.contains("catch(...)") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_CPP_CATCH_ALL,
                "检测到 catch (...)，建议只捕获明确异常类型并保留错误上下文",
            ));
        }

        if uses_cpp_endl(trimmed, &declared_names) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_CPP_STD_ENDL,
                "检测到 endl，若不需要强制刷新缓冲区，建议使用 '\\n'",
            ));
        }

        if contains_word(trimmed, "NULL") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_CPP_NULL_LITERAL,
                "检测到 NULL，现代 C++ 建议使用 nullptr 表达空指针",
            ));
        }
    }

    issues
}

fn missing_space_after_keyword(line: &str, keyword: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(keyword)
        && trimmed
            .as_bytes()
            .get(keyword.len())
            .is_some_and(|byte| *byte == b'(')
}

fn looks_like_c_style_cast(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#')
        || trimmed.starts_with("catch")
        || trimmed.starts_with("if")
        || trimmed.starts_with("while")
        || trimmed.starts_with("switch")
    {
        return false;
    }

    [
        "(int)",
        "(long)",
        "(float)",
        "(double)",
        "(char *)",
        "(void *)",
        "(const char *)",
    ]
    .iter()
    .any(|cast| trimmed.contains(cast))
}

fn check_rust_text_rules(
    file: &str,
    language: Language,
    lines: &[&str],
    original_lines: &[&str],
) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();

        if rust_call_or_macro(trimmed, "unwrap")
            || rust_call_or_macro(trimmed, "expect")
            || rust_call_or_macro(trimmed, "panic")
            || rust_call_or_macro(trimmed, "todo")
            || rust_call_or_macro(trimmed, "unimplemented")
        {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_RUST_PANIC_UNWRAP,
                "检测到 unwrap/expect/panic/todo/unimplemented，建议显式处理错误或限制到测试代码",
            ));
        }

        if contains_word(trimmed, "unsafe") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_RUST_UNSAFE,
                "检测到 unsafe 使用，建议收敛边界并补充 Safety 说明",
            ));

            if !has_nearby_safety_comment(original_lines, index) {
                issues.push(issue(
                    file,
                    language,
                    line_no,
                    Severity::Info,
                    RULE_RUST_UNSAFE_WITHOUT_SAFETY,
                    "unsafe 附近未看到 Safety 说明，建议写清前置条件和调用方责任",
                ));
            }
        }

        if trimmed.starts_with("#[allow(") || trimmed.starts_with("#![allow(") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_RUST_ALLOW_ATTR,
                "检测到 allow 属性，建议确认是否有明确原因和最小作用域",
            ));
        }

        if trimmed.contains("dbg!(") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_RUST_DBG_MACRO,
                "检测到 dbg! 调试宏，提交前建议改成日志或移除",
            ));
        }

        if trimmed.contains(".clone()") || trimmed.contains(".to_owned()") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_RUST_CLONE_NOISE,
                "检测到 clone/to_owned，建议确认是否真的需要分配或复制所有权",
            ));
        }

        if trimmed.contains("Rc<RefCell<") || trimmed.contains("Arc<Mutex<") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_RUST_COMPLEX_TYPE,
                "检测到 Rc<RefCell> 或 Arc<Mutex> 组合，建议确认共享可变状态边界清晰",
            ));
        }

        if contains_word(trimmed, "transmute") {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Warning,
                RULE_RUST_TRANSMUTE,
                "检测到 transmute，建议优先使用安全转换 API 或把 unsafe 边界压到最小",
            ));
        }

        if rust_as_cast(trimmed) {
            issues.push(issue(
                file,
                language,
                line_no,
                Severity::Info,
                RULE_RUST_AS_CAST,
                "检测到 as 转换，建议确认截断、符号位和平台相关行为是否符合预期",
            ));
        }
    }
    issues
}

fn has_nearby_safety_comment(lines: &[&str], index: usize) -> bool {
    let start = index.saturating_sub(3);
    lines[start..=index]
        .iter()
        .any(|line| line.contains("Safety") || line.contains("SAFETY"))
}

fn rust_as_cast(line: &str) -> bool {
    if line.starts_with("use ") || line.starts_with("//") {
        return false;
    }
    line.contains(" as ")
}

fn rust_call_or_macro(line: &str, name: &str) -> bool {
    let mut offset = 0;
    while let Some(pos) = line[offset..].find(name) {
        let start = offset + pos;
        if is_boundary(line, start, name.len()) {
            let cursor = skip_spaces(line, start + name.len());
            if line[cursor..].starts_with('(') || line[cursor..].starts_with("!(") {
                return true;
            }
        }
        offset = start + name.len();
    }
    false
}

fn uses_cpp_endl(line: &str, declared_names: &[String]) -> bool {
    line.contains("std::endl")
        || (!declared_names.iter().any(|name| name == "endl") && line.contains("<< endl"))
}

fn check_unused_includes(file: &str, language: Language, lines: &[&str]) -> Vec<crate::Issue> {
    let includes = collect_includes(lines);
    let code_without_includes = lines
        .iter()
        .filter(|line| !line.trim_start().starts_with("#include"))
        .copied()
        .collect::<Vec<_>>()
        .join("\n");

    let mut issues = Vec::new();
    for include in includes {
        if !header_looks_used(&include.header, &code_without_includes) {
            issues.push(issue(
                file,
                language,
                include.line,
                Severity::Warning,
                RULE_UNUSED_INCLUDE,
                format!("头文件 '{}' 可能未被使用", include.header),
            ));
        }
    }
    issues
}

fn check_unused_variables(file: &str, language: Language, lines: &[&str]) -> Vec<crate::Issue> {
    let declarations = lines
        .iter()
        .enumerate()
        .flat_map(|(index, line)| {
            extract_variable_declarations(line)
                .into_iter()
                .map(move |name| VarDecl {
                    line: index + 1,
                    name,
                })
        })
        .collect::<Vec<_>>();

    let code = lines.join("\n");
    let mut issues = Vec::new();
    for declaration in declarations {
        if declaration.name.starts_with('_') {
            continue;
        }
        if count_word_occurrences(&code, &declaration.name) <= 1 {
            issues.push(issue(
                file,
                language,
                declaration.line,
                Severity::Warning,
                RULE_UNUSED_VARIABLE,
                format!("变量 '{}' 定义后未使用", declaration.name),
            ));
        }
    }
    issues
}

fn check_function_metrics(file: &str, language: Language, lines: &[&str]) -> Vec<crate::Issue> {
    let mut issues = Vec::new();
    let mut brace_depth = 0usize;
    let mut function: Option<FunctionState> = None;
    let mut pending_header = String::new();
    let mut deep_nesting_reported = false;

    for (index, line) in lines.iter().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();
        let mut saw_top_level_open = false;

        let chars: Vec<char> = line.chars().collect();
        for (pos, ch) in chars.iter().enumerate() {
            match ch {
                '{' => {
                    if brace_depth == 0 {
                        let prefix = line[..byte_index(line, pos)].trim();
                        let header = if pending_header.is_empty() {
                            prefix.to_string()
                        } else if prefix.is_empty() {
                            pending_header.clone()
                        } else {
                            format!("{} {}", pending_header, prefix)
                        };
                        if looks_like_function_definition_header(&header) {
                            let name = extract_function_name(&header)
                                .unwrap_or_else(|| "<anonymous>".to_string());
                            function = Some(FunctionState {
                                name,
                                start_line: line_no,
                            });
                            deep_nesting_reported = false;
                        }
                        pending_header.clear();
                        saw_top_level_open = true;
                    }
                    brace_depth += 1;

                    if function.is_some() && brace_depth > 5 && !deep_nesting_reported {
                        issues.push(issue(
                            file,
                            language,
                            line_no,
                            Severity::Warning,
                            RULE_DEEP_NESTING,
                            "嵌套深度超过 4 层，建议重构",
                        ));
                        deep_nesting_reported = true;
                    }
                }
                '}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                    if brace_depth == 0 {
                        if let Some(state) = function.take() {
                            let function_lines = line_no.saturating_sub(state.start_line) + 1;
                            if function_lines > 100 {
                                issues.push(issue(
                                    file,
                                    language,
                                    state.start_line,
                                    Severity::Info,
                                    RULE_LONG_FUNCTION,
                                    format!(
                                        "函数 {}() 超过 100 行，当前 {} 行，建议拆分",
                                        state.name, function_lines
                                    ),
                                ));
                            }
                        }
                        pending_header.clear();
                    }
                }
                ';' if brace_depth == 0 => pending_header.clear(),
                _ => {}
            }
        }

        if brace_depth == 0 && trimmed.ends_with(';') {
            pending_header.clear();
        } else if brace_depth == 0
            && !saw_top_level_open
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            if !pending_header.is_empty() {
                pending_header.push(' ');
            }
            pending_header.push_str(trimmed);
        }
    }

    issues
}

fn sanitize_source(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        LineComment,
        BlockComment,
        StringLiteral,
        CharLiteral,
    }

    let mut output = String::with_capacity(source.len());
    let mut state = State::Normal;
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '/' if chars.peek() == Some(&'/') => {
                    output.push(' ');
                    output.push(' ');
                    chars.next();
                    state = State::LineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    output.push(' ');
                    output.push(' ');
                    chars.next();
                    state = State::BlockComment;
                }
                '"' => {
                    output.push(' ');
                    state = State::StringLiteral;
                }
                '\'' => {
                    output.push(' ');
                    state = State::CharLiteral;
                }
                _ => output.push(ch),
            },
            State::LineComment => {
                if ch == '\n' {
                    output.push('\n');
                    state = State::Normal;
                } else {
                    output.push(' ');
                }
            }
            State::BlockComment => {
                if ch == '*' && chars.peek() == Some(&'/') {
                    output.push(' ');
                    output.push(' ');
                    chars.next();
                    state = State::Normal;
                } else if ch == '\n' {
                    output.push('\n');
                } else {
                    output.push(' ');
                }
            }
            State::StringLiteral => {
                if ch == '\\' {
                    output.push(' ');
                    if let Some(next) = chars.next() {
                        if next == '\n' {
                            output.push('\n');
                        } else {
                            output.push(' ');
                        }
                    }
                } else if ch == '"' {
                    output.push(' ');
                    state = State::Normal;
                } else if ch == '\n' {
                    output.push('\n');
                    state = State::Normal;
                } else {
                    output.push(' ');
                }
            }
            State::CharLiteral => {
                if ch == '\\' {
                    output.push(' ');
                    if let Some(next) = chars.next() {
                        if next == '\n' {
                            output.push('\n');
                        } else {
                            output.push(' ');
                        }
                    }
                } else if ch == '\'' {
                    output.push(' ');
                    state = State::Normal;
                } else if ch == '\n' {
                    output.push('\n');
                    state = State::Normal;
                } else {
                    output.push(' ');
                }
            }
        }
    }

    output
}

fn violates_function_brace_newline(line: &str) -> bool {
    let Some(brace_index) = line.find('{') else {
        return false;
    };
    looks_like_function_definition_header(line[..brace_index].trim())
}

fn looks_like_function_definition_header(header: &str) -> bool {
    let header = header.trim();
    if header.is_empty()
        || header.starts_with('#')
        || header.ends_with(';')
        || header.contains('=')
        || !header.contains('(')
        || !header.contains(')')
    {
        return false;
    }

    let lower = header.to_ascii_lowercase();
    let leading_word = first_word(&lower).unwrap_or_default();
    if matches!(
        leading_word,
        "if" | "for" | "while" | "switch" | "else" | "do" | "case"
    ) {
        return false;
    }

    let close = header.rfind(')').unwrap_or(header.len());
    header[close + 1..].trim().is_empty()
}

fn extract_function_name(header: &str) -> Option<String> {
    let open = header.rfind('(')?;
    let before = header[..open].trim_end();
    let mut end = before.len();
    while end > 0 && before.as_bytes()[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 {
        let byte = before.as_bytes()[start - 1];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    (start < end).then(|| before[start..end].to_string())
}

fn single_line_for_has_braces(line: &str) -> bool {
    if find_keyword(line, "for").is_none() {
        return false;
    }
    let Some(brace_open) = line.find('{') else {
        return false;
    };
    let Some(brace_close) = line[brace_open + 1..].find('}') else {
        return false;
    };
    let body = &line[brace_open + 1..brace_open + 1 + brace_close];
    !body.trim().is_empty()
}

fn contains_void_main_empty_params(line: &str) -> bool {
    let mut offset = 0;
    while let Some(pos) = line[offset..].find("void") {
        let start = offset + pos;
        if !is_boundary(line, start, 4) {
            offset = start + 4;
            continue;
        }
        let mut cursor = start + 4;
        cursor = skip_spaces(line, cursor);
        if !line[cursor..].starts_with("main") || !is_boundary(line, cursor, 4) {
            offset = start + 4;
            continue;
        }
        cursor += 4;
        cursor = skip_spaces(line, cursor);
        if !line[cursor..].starts_with('(') {
            offset = start + 4;
            continue;
        }
        cursor += 1;
        cursor = skip_spaces(line, cursor);
        if line[cursor..].starts_with(')') {
            return true;
        }
        offset = start + 4;
    }
    false
}

fn control_statement_has_empty_body(line: &str, keyword: &str) -> bool {
    let Some((_, close)) = control_condition_span(line, keyword) else {
        return false;
    };
    let after = line[close + 1..].trim_start();
    after.starts_with(';')
}

fn condition_has_assignment(line: &str, keyword: &str) -> bool {
    let Some((open, close)) = control_condition_span(line, keyword) else {
        return false;
    };
    contains_single_assignment(&line[open + 1..close])
}

fn for_condition_has_assignment(line: &str) -> bool {
    let Some((open, close)) = control_condition_span(line, "for") else {
        return false;
    };
    let condition = &line[open + 1..close];
    let parts = split_top_level_semicolons(condition);
    parts
        .get(1)
        .is_some_and(|middle| contains_single_assignment(middle))
}

fn equality_used_as_statement(line: &str) -> bool {
    if line.starts_with("if")
        || line.starts_with("while")
        || line.starts_with("for")
        || line.starts_with("return")
        || line.starts_with('#')
        || !line.ends_with(';')
    {
        return false;
    }
    line.contains("==") && !contains_single_assignment(line)
}

fn control_condition_span(line: &str, keyword: &str) -> Option<(usize, usize)> {
    let keyword_pos = find_keyword(line, keyword)?;
    let open = line[keyword_pos + keyword.len()..].find('(')? + keyword_pos + keyword.len();
    let mut depth = 0usize;
    for (index, ch) in line[open..].char_indices() {
        let absolute = open + index;
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((open, absolute));
                }
            }
            _ => {}
        }
    }
    None
}

fn contains_single_assignment(expr: &str) -> bool {
    let bytes = expr.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'=' {
            continue;
        }
        let previous = index.checked_sub(1).and_then(|i| bytes.get(i)).copied();
        let next = bytes.get(index + 1).copied();
        if matches!(
            previous,
            Some(b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^')
        ) || matches!(next, Some(b'='))
        {
            continue;
        }
        return true;
    }
    false
}

fn split_top_level_semicolons(expr: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    for (index, ch) in expr.char_indices() {
        match ch {
            '(' => paren += 1,
            ')' => paren = paren.saturating_sub(1),
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            '{' => brace += 1,
            '}' => brace = brace.saturating_sub(1),
            ';' if paren == 0 && bracket == 0 && brace == 0 => {
                parts.push(expr[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(expr[start..].trim());
    parts
}

fn collect_includes(lines: &[&str]) -> Vec<IncludeDecl> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            parse_include(line).map(|header| IncludeDecl {
                line: index + 1,
                header,
            })
        })
        .collect()
}

fn parse_include(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("#include")?.trim();
    if let Some(rest) = rest.strip_prefix('<') {
        let closing = rest.find('>')?;
        Some(rest[..closing].to_string())
    } else if let Some(rest) = rest.strip_prefix('"') {
        let closing = rest.find('"')?;
        Some(rest[..closing].to_string())
    } else {
        None
    }
}

fn header_looks_used(header: &str, code: &str) -> bool {
    let symbols = header_symbols(header);
    if !symbols.is_empty() {
        return symbols.iter().any(|symbol| contains_word(code, symbol));
    }

    let stem = Path::new(header)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(header);
    contains_word(code, stem)
}

fn header_symbols(header: &str) -> &'static [&'static str] {
    match header {
        "stdio.h" => &[
            "printf", "fprintf", "sprintf", "snprintf", "scanf", "fscanf", "sscanf", "puts",
            "fputs", "putchar", "getchar", "fgets", "FILE", "fopen", "fclose", "perror",
        ],
        "stdlib.h" => &[
            "malloc", "calloc", "realloc", "free", "exit", "atoi", "atol", "atof", "strtol",
            "strtoul", "rand", "srand",
        ],
        "string.h" => &[
            "strlen", "strcpy", "strncpy", "strcmp", "strncmp", "strcat", "strncat", "strstr",
            "strchr", "strrchr", "memset", "memcpy", "memmove", "memcmp",
        ],
        "math.h" => &[
            "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "pow", "floor", "ceil", "fabs",
            "log", "exp",
        ],
        "ctype.h" => &[
            "isalnum", "isalpha", "isdigit", "islower", "isupper", "isspace", "tolower", "toupper",
        ],
        "stdbool.h" => &["bool", "true", "false"],
        "stdint.h" => &[
            "int8_t",
            "int16_t",
            "int32_t",
            "int64_t",
            "uint8_t",
            "uint16_t",
            "uint32_t",
            "uint64_t",
            "intptr_t",
            "uintptr_t",
        ],
        "stddef.h" => &["size_t", "ptrdiff_t", "NULL", "offsetof"],
        _ => &[],
    }
}

fn extract_variable_declarations(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || !trimmed.ends_with(';')
        || trimmed.contains('(')
        || starts_with_any(
            trimmed,
            &[
                "return", "if", "for", "while", "switch", "case", "break", "continue",
            ],
        )
    {
        return Vec::new();
    }

    let without_semicolon = trimmed.trim_end_matches(';').trim();
    let Some(declarators_start) = find_declarators_start(without_semicolon) else {
        return Vec::new();
    };
    let declarators = without_semicolon[declarators_start..].trim();
    if declarators.is_empty() {
        return Vec::new();
    }

    split_top_level_commas(declarators)
        .into_iter()
        .filter_map(extract_declarator_name)
        .collect()
}

fn find_declarators_start(declaration: &str) -> Option<usize> {
    let mut cursor = 0usize;
    let mut saw_type = false;

    while let Some((word, start, end)) = read_word_at(declaration, cursor) {
        if start != cursor && !declaration[cursor..start].trim().is_empty() {
            break;
        }

        if is_type_qualifier(word) {
            cursor = end;
            cursor = skip_spaces(declaration, cursor);
            continue;
        }

        if matches!(word, "struct" | "enum" | "union") {
            saw_type = true;
            cursor = end;
            cursor = skip_spaces(declaration, cursor);
            if let Some((_, _, tag_end)) = read_word_at(declaration, cursor) {
                cursor = tag_end;
                cursor = skip_spaces(declaration, cursor);
            }
            continue;
        }

        if is_type_word(word) {
            saw_type = true;
            cursor = end;
            cursor = skip_spaces(declaration, cursor);
            continue;
        }

        break;
    }

    saw_type.then_some(cursor)
}

fn is_type_qualifier(word: &str) -> bool {
    matches!(
        word,
        "auto" | "const" | "extern" | "register" | "restrict" | "static" | "volatile" | "inline"
    )
}

fn is_type_word(word: &str) -> bool {
    matches!(
        word,
        "char"
            | "double"
            | "float"
            | "int"
            | "long"
            | "short"
            | "signed"
            | "unsigned"
            | "void"
            | "bool"
            | "size_t"
            | "ssize_t"
            | "FILE"
    ) || word.ends_with("_t")
}

fn split_top_level_commas(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;

    for (index, ch) in value.char_indices() {
        match ch {
            '(' => paren += 1,
            ')' => paren = paren.saturating_sub(1),
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            '{' => brace += 1,
            '}' => brace = brace.saturating_sub(1),
            ',' if paren == 0 && bracket == 0 && brace == 0 => {
                parts.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    parts.push(value[start..].trim());
    parts
}

fn extract_declarator_name(declarator: &str) -> Option<String> {
    let before_initializer = declarator
        .split_once('=')
        .map(|(left, _)| left)
        .unwrap_or(declarator)
        .trim();
    let value = before_initializer.trim_start_matches('*').trim();
    if value.starts_with('(') {
        return None;
    }

    let mut name = String::new();
    for ch in value.chars() {
        if name.is_empty() {
            if ch == '_' || ch.is_ascii_alphabetic() {
                name.push(ch);
            } else if ch == '*' || ch.is_ascii_whitespace() {
                continue;
            } else {
                return None;
            }
        } else if ch == '_' || ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else {
            break;
        }
    }

    (!name.is_empty() && !is_c_keyword(&name)).then_some(name)
}

fn count_word_occurrences(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut offset = 0;
    while let Some(pos) = haystack[offset..].find(needle) {
        let start = offset + pos;
        if is_boundary(haystack, start, needle.len()) {
            count += 1;
        }
        offset = start + needle.len();
    }
    count
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    count_word_occurrences(haystack, needle) > 0
}

fn find_keyword(haystack: &str, keyword: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(pos) = haystack[offset..].find(keyword) {
        let start = offset + pos;
        if is_boundary(haystack, start, keyword.len()) {
            return Some(start);
        }
        offset = start + keyword.len();
    }
    None
}

fn is_boundary(haystack: &str, start: usize, len: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| haystack.as_bytes().get(index))
        .copied();
    let after = haystack.as_bytes().get(start + len).copied();
    !before.is_some_and(is_ident_byte) && !after.is_some_and(is_ident_byte)
}

fn is_ident_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn skip_spaces(value: &str, mut cursor: usize) -> usize {
    while value
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        cursor += 1;
    }
    cursor
}

fn read_word_at(value: &str, cursor: usize) -> Option<(&str, usize, usize)> {
    let bytes = value.as_bytes();
    let mut start = cursor;
    while bytes
        .get(start)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        start += 1;
    }
    let first = *bytes.get(start)?;
    if !(first == b'_' || first.is_ascii_alphabetic()) {
        return None;
    }
    let mut end = start + 1;
    while bytes.get(end).is_some_and(|byte| is_ident_byte(*byte)) {
        end += 1;
    }
    Some((&value[start..end], start, end))
}

fn first_word(value: &str) -> Option<&str> {
    read_word_at(value, 0).map(|(word, _, _)| word)
}

fn starts_with_any(value: &str, words: &[&str]) -> bool {
    words
        .iter()
        .any(|word| value.starts_with(word) && is_boundary(value, 0, word.len()))
}

fn is_c_keyword(value: &str) -> bool {
    matches!(
        value,
        "auto"
            | "break"
            | "case"
            | "char"
            | "const"
            | "continue"
            | "default"
            | "do"
            | "double"
            | "else"
            | "enum"
            | "extern"
            | "float"
            | "for"
            | "goto"
            | "if"
            | "inline"
            | "int"
            | "long"
            | "register"
            | "restrict"
            | "return"
            | "short"
            | "signed"
            | "sizeof"
            | "static"
            | "struct"
            | "switch"
            | "typedef"
            | "union"
            | "unsigned"
            | "void"
            | "volatile"
            | "while"
    )
}

fn byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

fn is_cpp_header(file: &str) -> bool {
    Path::new(file)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "hpp" | "hxx" | "h"))
}
