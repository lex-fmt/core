use txxt::txxt::lexers::transformations::_lex;
use txxt::txxt::lexers::LineTokenTree;

#[test]
fn test_definition_with_blank_line_after() {
    // Definition followed by blank line and another element
    let source = "Definition:\n    Content\n\nNext paragraph\n";
    println!("\nSource (visual):\n{}\n", source);

    let tree = _lex(source).expect("Failed to tokenize");

    fn print_tree(tree: &[LineTokenTree], indent: usize) {
        for (i, node) in tree.iter().enumerate() {
            match node {
                LineTokenTree::Token(token) => {
                    println!(
                        "{}[{}] Token: {:?}",
                        "  ".repeat(indent),
                        i,
                        token.line_type
                    );
                }
                LineTokenTree::Block(children) => {
                    println!("{}[{}] Block:", "  ".repeat(indent), i);
                    print_tree(children, indent + 1);
                }
            }
        }
    }

    println!("Token Tree:");
    print_tree(&tree, 0);

    // We should see:
    // [0] Token: SubjectLine (Definition:)
    // [1] Block (Content)
    // [2] Token: BlankLine
    // [3] Token: ParagraphLine (Next paragraph)
}
