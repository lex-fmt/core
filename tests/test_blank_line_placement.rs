use txxt::txxt::lexers::transformations::_lex;
use txxt::txxt::lexers::LineTokenTree;

#[test]
fn test_blank_line_placement() {
    let source = "Foo:\n\n    Bar\n";
    println!("\nSource:\n{:?}\n", source);
    println!("Source (visual):\n{}\n", source);

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
}
