//! Export tests (Lex → Markdown)

use lex_babel::formats::MarkdownFormat;
use lex_babel::Format;
use lex_parser::lex::transforms::standard::STRING_TO_AST;

#[test]
fn test_trifecta_020_multiple_sessions() {
    let lex_src = std::fs::read_to_string(
        "/Users/adebert/h/lex/docs/specs/v1/trifecta/020-paragraphs-sessions-flat-multiple.lex",
    )
    .expect("Failed to read trifecta 020 file");

    let lex_doc = STRING_TO_AST.run(lex_src).unwrap();
    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    println!("Generated Markdown:\n{}", md);

    // Check for multiple sessions
    assert!(md.contains("# 1. First Session"));
    assert!(md.contains("# 2. Second Session"));
    assert!(md.contains("# 3. Third Session"));
    assert!(md.contains("# 4. Session Without Numbering"));

    // Check for paragraphs
    assert!(md.contains("Multiple Sessions Flat Test"));
    assert!(md.contains("A paragraph between sessions"));
    assert!(md.contains("Final paragraph at the root level"));
}

#[test]
fn test_trifecta_060_nesting() {
    let lex_src = std::fs::read_to_string(
        "/Users/adebert/h/lex/docs/specs/v1/trifecta/060-trifecta-nesting.lex",
    )
    .expect("Failed to read trifecta 060 file");

    let lex_doc = STRING_TO_AST.run(lex_src).unwrap();
    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    println!("Generated Markdown:\n{}", md);

    // Check for nested sessions
    assert!(md.contains("# 1. Root Session"));
    assert!(md.contains("## 1.1. Sub-session with Paragraph"));
    assert!(md.contains("## 1.2. Sub-session with List"));
    assert!(md.contains("### 1.2.1. Deeply Nested Session"));

    // Check for lists
    assert!(md.contains("- Then has a list"));
    assert!(md.contains("- With multiple items"));
    assert!(md.contains("- Starts with a list"));

    // Check for mixed content
    assert!(md.contains("This root session contains various nested elements"));
    assert!(md.contains("Back to the root session level"));
}

#[test]
fn test_simple_list() {
    let lex_src = r#"Test:

- First item
- Second item
- Third item
"#;

    let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();

    // Debug: Check IR events
    let ir_doc = lex_babel::to_ir(&lex_doc);
    let events = lex_babel::mappings::nested_to_flat::tree_to_events(
        &lex_babel::ir::nodes::DocNode::Document(ir_doc),
    );
    println!("List Events ({} total):", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  [{}] {:?}", i, event);
    }

    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    println!("\nGenerated Markdown:\n{}", md);

    assert!(md.contains("- First item"));
    assert!(md.contains("- Second item"));
    assert!(md.contains("- Third item"));
}

#[test]
fn test_verbatim_block() {
    let lex_src = r#"Code Example:

    function hello() {
        return "world";
    }

:: javascript
"#;

    let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();
    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    println!("Generated Markdown:\n{}", md);

    // Check for code fence
    assert!(md.contains("```javascript"));
    assert!(md.contains("function hello()"));
    assert!(md.contains("return \"world\""));
    assert!(md.contains("```"));
}

#[test]
fn test_inline_formatting() {
    let lex_src = "This has *bold text* and _italic text_ and `code` inline.\n";

    let lex_doc = STRING_TO_AST.run(lex_src.to_string()).unwrap();
    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    println!("Generated Markdown:\n{}", md);

    assert!(md.contains("**bold text**"));
    assert!(md.contains("*italic text*"));
    assert!(md.contains("`code`"));
}

#[test]
fn test_kitchensink() {
    let lex_src =
        std::fs::read_to_string("/private/tmp/markdown/lex-babel/tests/fixtures/kitchensink.lex")
            .expect("Failed to read kitchensink file");

    let lex_doc = STRING_TO_AST.run(lex_src).unwrap();
    let format = MarkdownFormat;
    let result = format.serialize(&lex_doc);

    assert!(result.is_ok());
    let md = result.unwrap();

    // Use snapshot testing for comprehensive output
    insta::assert_snapshot!(md);
}
