use lex_analysis::semantic_tokens::{collect_semantic_tokens, LexSemanticTokenKind};
use lex_parser::lex::parsing;

fn main() {
    // Read from file if provided, otherwise use inline source
    let source = if let Some(path) = std::env::args().nth(1) {
        std::fs::read_to_string(&path).expect(&format!("Failed to read file: {}", path))
    } else {
        r#":: doc.note severity=info :: Document preface for semantic tokens coverage.

1. Intro

    Welcome to *Lex* _format_ with `code` and #math# plus references [^source] and [@spec2025 p.4] and [Cache].

    Cache:
        A definition body referencing [Cache].

    :: callout ::
        Session-level annotation body.
    ::

    - Bullet item referencing [1]
    - Nested bullet

        Nested paragraph inside list.

    CLI Example:
        lex build
        lex serve
    :: shell language=bash

See https://lexlang.org for docs.

2. Notes

1. Footnote forty two for bullet.
2. Footnote referenced in text.
"#.to_string()
    };

    let document = parsing::parse_document(&source).expect("Failed to parse");
    let tokens = collect_semantic_tokens(&document);

    println!("Found {} semantic tokens:\n", tokens.len());

    for token in &tokens {
        let snippet = &source[token.range.span.clone()];
        let line = snippet.lines().next().unwrap_or(snippet);
        let preview = if line.len() > 50 {
            format!("{}...", &line[..50])
        } else {
            line.to_string()
        };

        println!("{:30} | {:?}", format!("{:?}", token.kind), preview);
    }

    // Group by kind
    println!("\n\nGrouped by token kind:");
    println!("======================\n");

    let mut by_kind: std::collections::HashMap<LexSemanticTokenKind, Vec<String>> =
        std::collections::HashMap::new();

    for token in &tokens {
        let snippet = &source[token.range.span.clone()];
        by_kind
            .entry(token.kind)
            .or_insert_with(Vec::new)
            .push(snippet.to_string());
    }

    let mut kinds: Vec<_> = by_kind.keys().collect();
    kinds.sort_by_key(|k| format!("{:?}", k));

    for kind in kinds {
        println!("{:?}:", kind);
        for snippet in &by_kind[kind] {
            let line = snippet.lines().next().unwrap_or(snippet);
            println!("  - {:?}", line);
        }
        println!();
    }
}
