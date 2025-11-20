//! CLI-specific transforms
//!
//! This module defines all the transform combinations available in the CLI.
//! Each transform is a stage + format combination (e.g., "ast-tag", "token-core-json").

use lex_babel::formats::{tag::serialize_document as serialize_ast_tag, treeviz::to_treeviz_str};
use lex_parser::lex::lexing::transformations::line_token_grouping::GroupedTokens;
use lex_parser::lex::lexing::transformations::LineTokenGroupingMapper;
use lex_parser::lex::loader::DocumentLoader;
use lex_parser::lex::token::{to_line_container, LineContainer, LineToken};
use lex_parser::lex::transforms::standard::{CORE_TOKENIZATION, LEXING, TO_IR};
use std::collections::HashMap;

/// All available CLI transforms (stage + format combinations)
pub const AVAILABLE_TRANSFORMS: &[&str] = &[
    "token-core-json",
    "token-core-simple",
    "token-core-pprint",
    "token-simple", // alias for token-core-simple
    "token-pprint", // alias for token-core-pprint
    "token-line-json",
    "token-line-simple",
    "token-line-pprint",
    "ir-json",
    "ast-json",
    "ast-tag",
    "ast-treeviz",
];

/// Execute a named transform on a source file with optional extra parameters
pub fn execute_transform(
    source: &str,
    transform_name: &str,
    extra_params: &HashMap<String, String>,
) -> Result<String, String> {
    let loader = DocumentLoader::from_string(source);

    match transform_name {
        "token-core-json" => {
            let tokens = loader
                .with(&CORE_TOKENIZATION)
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(serde_json::to_string_pretty(&tokens_to_json(&tokens))
                .map_err(|e| format!("JSON serialization failed: {}", e))?)
        }
        "token-core-simple" | "token-simple" => {
            let tokens = loader
                .with(&CORE_TOKENIZATION)
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(tokens_to_simple(&tokens))
        }
        "token-core-pprint" | "token-pprint" => {
            let tokens = loader
                .with(&CORE_TOKENIZATION)
                .map_err(|e| format!("Transform failed: {}", e))?;
            Ok(tokens_to_pprint(&tokens))
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
        "token-line-simple" => {
            let tokens = loader
                .with(&LEXING)
                .map_err(|e| format!("Transform failed: {}", e))?;
            let mut mapper = LineTokenGroupingMapper::new();
            let grouped = mapper.map(tokens);
            let line_tokens: Vec<LineToken> = grouped
                .into_iter()
                .map(GroupedTokens::into_line_token)
                .collect();
            Ok(line_tokens_to_simple(&line_tokens))
        }
        "token-line-pprint" => {
            let tokens = loader
                .with(&LEXING)
                .map_err(|e| format!("Transform failed: {}", e))?;
            let mut mapper = LineTokenGroupingMapper::new();
            let grouped = mapper.map(tokens);
            let line_tokens: Vec<LineToken> = grouped
                .into_iter()
                .map(GroupedTokens::into_line_token)
                .collect();
            Ok(line_tokens_to_pprint(&line_tokens))
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
            // TODO: Pass extra_params to to_treeviz_str when API supports it
            // For example: to_treeviz_str(&doc, extra_params)
            // This would allow params like: --extra-all-nodes true
            if !extra_params.is_empty() {
                eprintln!(
                    "Note: Extra parameters received but not yet supported by ast-treeviz: {:?}",
                    extra_params
                );
            }
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

fn tokens_to_simple(tokens: &[(lex_parser::lex::token::Token, std::ops::Range<usize>)]) -> String {
    tokens
        .iter()
        .map(|(token, _)| token.simple_name())
        .collect::<Vec<_>>()
        .join("\n")
}

fn tokens_to_pprint(tokens: &[(lex_parser::lex::token::Token, std::ops::Range<usize>)]) -> String {
    use lex_parser::lex::token::Token;

    let mut output = String::new();
    for (token, _) in tokens {
        output.push_str(token.simple_name());
        output.push('\n');
        if matches!(token, Token::BlankLine(_)) {
            output.push('\n');
        }
    }
    output
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

fn line_tokens_to_simple(line_tokens: &[LineToken]) -> String {
    line_tokens
        .iter()
        .map(|line| line.line_type.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_tokens_to_pprint(line_tokens: &[LineToken]) -> String {
    let container = to_line_container::build_line_container(line_tokens.to_vec());
    let mut output = String::new();
    render_line_tree(&container, 0, true, &mut output);
    output
}

fn render_line_tree(node: &LineContainer, depth: usize, is_root: bool, output: &mut String) {
    match node {
        LineContainer::Token(line) => {
            let indent = "  ".repeat(depth);
            output.push_str(&indent);
            output.push_str(&line.line_type.to_string());
            output.push('\n');
        }
        LineContainer::Container { children } => {
            let next_depth = if is_root { depth } else { depth + 1 };
            for child in children {
                render_line_tree(child, next_depth, false, output);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_line_transform_emits_line_tokens() {
        let source = "Session:\n    Content\n";
        let extra_params = HashMap::new();
        let output =
            execute_transform(source, "token-line-json", &extra_params).expect("transform to run");

        assert!(output.contains("\"line_type\""));
        assert!(output.contains("SubjectLine"));
        assert!(output.contains("ParagraphLine"));
    }

    #[test]
    fn token_simple_outputs_names() {
        let source = "Session:\n    Content\n";
        let extra_params = HashMap::new();
        let output =
            execute_transform(source, "token-simple", &extra_params).expect("transform to run");

        assert!(output.contains("TEXT"));
        assert!(output.contains("BLANK_LINE"));
    }

    #[test]
    fn token_line_simple_outputs_names() {
        let source = "Session:\n    Content\n";
        let extra_params = HashMap::new();
        let output = execute_transform(source, "token-line-simple", &extra_params)
            .expect("transform to run");

        assert!(output.contains("SUBJECT_LINE"));
        assert!(output.contains("PARAGRAPH_LINE"));
    }

    #[test]
    fn token_pprint_inserts_blank_line() {
        let source = "Hello\n\nWorld\n";
        let extra_params = HashMap::new();
        let output =
            execute_transform(source, "token-pprint", &extra_params).expect("transform to run");

        assert!(output.contains("BLANK_LINE\n\n"));
    }

    #[test]
    fn token_line_pprint_indents_children() {
        let source = "Session:\n    Content\n";
        let extra_params = HashMap::new();
        let output = execute_transform(source, "token-line-pprint", &extra_params)
            .expect("transform to run");

        assert!(output.contains("SUBJECT_LINE"));
        assert!(output.contains("  PARAGRAPH_LINE"));
    }

    #[test]
    fn execute_transform_accepts_extra_params() {
        let source = "# Test\n";
        let mut extra_params = HashMap::new();
        extra_params.insert("all-nodes".to_string(), "true".to_string());
        extra_params.insert("max-depth".to_string(), "5".to_string());

        // Should not error, even though params aren't used yet
        let result = execute_transform(source, "ast-treeviz", &extra_params);
        assert!(result.is_ok());
    }
}
