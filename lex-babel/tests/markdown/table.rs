use lex_babel::format::Format;
use lex_babel::formats::markdown::MarkdownFormat;

#[test]
fn test_table_round_trip() {
    let md = r#"| Header 1 | Header 2 |
| :--- | :---: |
| Cell 1 | Cell 2 |
| Cell 3 | Cell 4 |
"#;
    // Note: Comrak might normalize whitespace/alignment chars.

    // Markdown -> Lex
    let doc = MarkdownFormat.parse(md).expect("Failed to parse markdown");

    // Lex -> Markdown
    let output = MarkdownFormat
        .serialize(&doc)
        .expect("Failed to serialize markdown");

    println!("Original:\n{}", md);
    println!("Output:\n{}", output);

    // Verify content presence
    assert!(output.contains("| Header 1 | Header 2 |"));
    assert!(output.contains("Cell 1"));
    assert!(output.contains("Cell 2"));

    // Verify alignment markers exist (Comrak output format)
    // Comrak usually outputs `| :--- | :---: |` or similar.
    assert!(output.contains(":--"));
    assert!(output.contains(":-:"));
}
