use lex::lex::lexers::linebased::transformations::unwrap_container_to_token_tree;
use lex::lex::lexers::transformations::_lex;
use lex::lex::lexers::LineTokenTree;

#[test]
fn test_list_with_blank_line_before_item_content() {
    // List item with blank line before its content block
    let source = "- First item\n\n    Content of first item\n- Second item\n";
    println!("\nSource (visual):\n{}\n", source);

    let container = _lex(source).expect("Failed to tokenize");
    let tree = unwrap_container_to_token_tree(&container);

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
                LineTokenTree::Container(_) => {
                    println!("{}[{}] Container", "  ".repeat(indent), i);
                }
            }
        }
    }

    println!("Token Tree:");
    print_tree(&tree, 0);

    // Expected structure:
    // [0] Token: ListLine (- First item)
    // [1] Token: BlankLine
    // [2] Block (Content)
    // [3] Token: ListLine (- Second item)
}
