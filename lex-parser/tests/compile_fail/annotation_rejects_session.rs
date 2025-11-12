use lex_parser::lex::ast::elements::typed_content::SessionContent;
use lex_parser::lex::ast::{Annotation, Label, Session};

fn main() {
    let label = Label::new("note".to_string());
    let session = Session::with_title("Nested".to_string());
    let _annotation = Annotation::new(label, vec![], vec![SessionContent::Session(session)]);
}
