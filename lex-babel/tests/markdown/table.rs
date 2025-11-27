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

#[test]
fn test_table_alignment_import() {
    use lex_parser::lex::ast::elements::Annotation;
    use lex_parser::lex::ast::ContentItem;

    let md = r#"| Left | Center | Right |
| :--- | :----: | ----: |
| L    | C      | R     |
"#;

    let doc = MarkdownFormat.parse(md).expect("Failed to parse markdown");

    // Traverse AST to find table cells and check alignment parameters
    // Structure: Document -> Table (Annotation) -> TBody (Annotation) -> TR (Annotation) -> TD (Annotation)

    // Helper to find annotation by label
    fn find_annotation<'a>(items: &'a [ContentItem], label: &str) -> Option<&'a Annotation> {
        for item in items {
            if let ContentItem::Annotation(ann) = item {
                if ann.data.label.value == label {
                    return Some(ann);
                }
                // Search children
                if let Some(found) = find_annotation(&ann.children, label) {
                    return Some(found);
                }
            }
        }
        None
    }

    // We expect to find cells with specific alignment parameters
    let root_children = &doc.root.children;

    // This is a bit manual, but we want to verify the structure deep down
    // 1. Find Table
    let table = find_annotation(root_children, "table").expect("Should have table");

    // 2. Find TBody
    let tbody = find_annotation(&table.children, "tbody").expect("Should have tbody");

    // 3. Find first TR
    let tr = find_annotation(&tbody.children, "tr").expect("Should have tr");

    // 4. Check cells
    let cells: Vec<&Annotation> = tr
        .children
        .iter()
        .filter_map(|c| {
            if let ContentItem::Annotation(ann) = c {
                Some(ann)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(cells.len(), 3);

    // Check Left
    let left = cells[0];
    let left_align = left
        .data
        .parameters
        .iter()
        .find(|p| p.key == "align")
        .map(|p| p.value.as_str());
    assert_eq!(left_align, Some("left"));

    // Check Center
    let center = cells[1];
    let center_align = center
        .data
        .parameters
        .iter()
        .find(|p| p.key == "align")
        .map(|p| p.value.as_str());
    assert_eq!(center_align, Some("center"));

    // Check Right
    let right = cells[2];
    let right_align = right
        .data
        .parameters
        .iter()
        .find(|p| p.key == "align")
        .map(|p| p.value.as_str());
    assert_eq!(right_align, Some("right"));
}
