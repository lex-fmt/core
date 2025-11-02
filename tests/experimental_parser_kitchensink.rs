//! Integration test for experimental parser using the kitchensink test file.
//!
//! This test uses insta snapshot testing to ensure the experimental parser
//! produces the correct AST structure for a complex, comprehensive test file.
//! Any regression in parsing will be caught automatically.

use txxt::txxt::lexer::transformations::experimental_lex;
use txxt::txxt::parser::homy::engine::parse_experimental;
use txxt::txxt::parser::ContentItem;

#[test]
fn experimental_parser_kitchensink_snapshot() {
    let source = std::fs::read_to_string("docs/specs/v1/regression-bugs/kitchensink.txxt")
        .expect("Could not read kitchensink.txxt");

    let tree = experimental_lex(&source).expect("Failed to tokenize");
    let doc = parse_experimental(tree, &source).expect("Parser failed");

    // Create a readable representation of the AST for snapshot testing
    let snapshot = format_ast_snapshot(&doc.root.content);
    insta::assert_snapshot!(snapshot);
}

/// Format the AST into a readable structure for snapshot testing
fn format_ast_snapshot(content: &[ContentItem]) -> String {
    let mut output = String::new();
    output.push_str(&format!("Document with {} root items:\n\n", content.len()));

    for (i, item) in content.iter().enumerate() {
        output.push_str(&format!("[{}] {}\n", i, format_item(item, 0)));
    }

    output
}

fn format_item(item: &ContentItem, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match item {
        ContentItem::Paragraph(p) => {
            format!(
                "Paragraph with {} line(s): {}",
                p.lines.len(),
                format_lines(&p.lines, indent + 1)
            )
        }
        ContentItem::Session(s) => {
            let mut result = format!("Session with {} item(s):\n", s.content.len());
            for (j, sub_item) in s.content.iter().enumerate() {
                result.push_str(&format!(
                    "{}  [{}] {}\n",
                    prefix,
                    j,
                    format_item(sub_item, indent + 1)
                ));
            }
            result.trim_end().to_string()
        }
        ContentItem::List(l) => {
            let mut result = format!("List with {} item(s):\n", l.content.len());
            for (j, list_item) in l.content.iter().enumerate() {
                if let ContentItem::ListItem(li) = list_item {
                    result.push_str(&format!(
                        "{}  [{}] List item with {} content item(s):\n",
                        prefix,
                        j,
                        li.content.len()
                    ));
                    for (k, sub_item) in li.content.iter().enumerate() {
                        result.push_str(&format!(
                            "{}    [{}] {}\n",
                            prefix,
                            k,
                            format_item(sub_item, indent + 2)
                        ));
                    }
                } else {
                    result.push_str(&format!(
                        "{}  [{}] {}\n",
                        prefix,
                        j,
                        format_item(list_item, indent + 1)
                    ));
                }
            }
            result.trim_end().to_string()
        }
        ContentItem::Definition(d) => {
            let mut result = format!("Definition with {} item(s):\n", d.content.len());
            for (j, sub_item) in d.content.iter().enumerate() {
                result.push_str(&format!(
                    "{}  [{}] {}\n",
                    prefix,
                    j,
                    format_item(sub_item, indent + 1)
                ));
            }
            result.trim_end().to_string()
        }
        ContentItem::Annotation(a) => {
            let mut result = format!(
                "Annotation with {} parameter(s) and {} content item(s):\n",
                a.parameters.len(),
                a.content.len()
            );
            if !a.content.is_empty() {
                for (j, sub_item) in a.content.iter().enumerate() {
                    result.push_str(&format!(
                        "{}  [{}] {}\n",
                        prefix,
                        j,
                        format_item(sub_item, indent + 1)
                    ));
                }
            }
            result.trim_end().to_string()
        }
        ContentItem::ForeignBlock(fb) => {
            format!("ForeignBlock with {} content line(s)", fb.content.len())
        }
        ContentItem::ListItem(li) => {
            format!("ListItem with {} content item(s)", li.content.len())
        }
        ContentItem::TextLine(tl) => {
            format!("TextLine: {}", tl.content.as_string())
        }
    }
}

fn format_lines(lines: &[ContentItem], indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!(
            "{}[{}] {}",
            prefix,
            i,
            format_item(line, indent + 1)
        ));
    }

    if lines.len() > 1 {
        format!("\n{}", result)
    } else {
        result
    }
}
