use tree_sitter::{Parser, Tree};

use crate::model::Language;

pub fn parse_tree(language: Language, source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = match language {
        Language::C => tree_sitter_c::LANGUAGE.into(),
        Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
    };
    parser.set_language(&language).ok()?;
    parser.parse(source, None)
}
