//! CLI-specific transforms
//!
//! This module defines all the transform combinations available in the CLI.
//! Each transform is a stage + format combination (e.g., "ast-tag", "token-core-json").

use lex_parser::lex::formats::{serialize_ast_tag, to_treeviz_str};
use lex_parser::lex::lexing::transformations::line_token_grouping::GroupedTokens;
use lex_parser::lex::lexing::transformations::LineTokenGroupingMapper;
use lex_parser::lex::loader::DocumentLoader;
use lex_parser::lex::token::LineToken;
use lex_parser::lex::transforms::standard::{CORE_TOKENIZATION, LEXING, TO_IR};

/// All available CLI transforms (stage + format combinations)
pub const AVAILABLE_TRANSFORMS: &[&str] = &[
    "token-core-json",
    "token-line-json",
    "ir-json",
    "ast-json",
    "ast-tag",
    "ast-treeviz",
];

/// Execute a named transform on a source file
pub fn execute_transform(source: &str, transform_name: &str) -> Result<String, String> {
    let loader = DocumentLoader::from_string(source);

    match transform_name {
        "token-core-json" => {
            let tokens = loader
                .with(&CORE_TOKENIZATION)
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(serde_json::to_string_pretty(&tokens_to_json(&tokens))
                .map_err(|e| format!("JSON serialization failed: {}", e))?)
        }
        "token-line-json" => {
            let tokens = loader
                .with(&LEXING)
                .map_err(|e| format!("Transform failed: {}", e))?;
            let mut mapper = LineTokenGroupingMapper::new();
            let grouped = mapper.map(tokens);
            let line_tokens: Vec<LineToken> = grouped
                .into_iter()
                .map(GroupedTokens::into_line_token)
                .collect();
            Ok(
                serde_json::to_string_pretty(&line_tokens_to_json(&line_tokens))
                    .map_err(|e| format!("JSON serialization failed: {}", e))?,
            )
        }
        "ir-json" => {
            let ir = loader
                .with(&TO_IR)
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(serde_json::to_string_pretty(&ir_to_json(&ir))
                .map_err(|e| format!("JSON serialization failed: {}", e))?)
        }
        "ast-json" => {
            let doc = loader
                .parse()
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(serde_json::to_string_pretty(&ast_to_json(&doc))
                .map_err(|e| format!("JSON serialization failed: {}", e))?)
        }
        "ast-tag" => {
            let doc = loader
                .parse()
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(serialize_ast_tag(&doc))
        }
        "ast-treeviz" => {
            let doc = loader
                .parse()
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(to_treeviz_str(&doc))
        }
        _ => Err(format!("Unknown transform: {}", transform_name)),
    }
}

/// Convert tokens to JSON-serializable format
fn tokens_to_json(
    tokens: &[(lex_parser::lex::token::Token, std::ops::Range<usize>)],
) -> serde_json::Value {
    use serde_json::json;

    json!(tokens
        .iter()
        .map(|(token, range)| {
            json!({
                "token": format!("{:?}", token),
                "start": range.start,
                "end": range.end,
            })
        })
        .collect::<Vec<_>>())
}

/// Convert line tokens into a JSON-friendly structure
fn line_tokens_to_json(line_tokens: &[LineToken]) -> serde_json::Value {
    use serde_json::json;

    json!(line_tokens
        .iter()
        .map(|line| {
            json!({
                "line_type": format!("{:?}", line.line_type),
                "tokens": line
                    .source_tokens
                    .iter()
                    .zip(line.token_spans.iter())
                    .map(|(token, span)| {
                        json!({
                            "token": format!("{:?}", token),
                            "start": span.start,
                            "end": span.end,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_line_transform_emits_line_tokens() {
        let source = "Session:\n    Content\n";
        let output = execute_transform(source, "token-line-json").expect("transform to run");

        assert!(output.contains("\"line_type\""));
        assert!(output.contains("SubjectLine"));
        assert!(output.contains("ParagraphLine"));
    }
}

/// Convert IR (ParseNode) to JSON-serializable format
fn ir_to_json(node: &lex_parser::lex::parsing::ir::ParseNode) -> serde_json::Value {
    use serde_json::json;

    json!({
        "type": format!("{:?}", node.node_type),
        "tokens": tokens_to_json(&node.tokens),
        "children": node.children.iter().map(ir_to_json).collect::<Vec<_>>(),
        "has_payload": node.payload.is_some(),
    })
}

/// Convert AST (Document) to JSON-serializable format
fn ast_to_json(doc: &lex_parser::lex::parsing::Document) -> serde_json::Value {
    use serde_json::json;

    json!({
        "type": "Document",
        "children_count": doc.root.children.len(),
        // For now, just a basic representation
        // Can be expanded to include full AST details
    })
}
