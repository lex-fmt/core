use lex_parser::lex::ast::elements::typed_content::SessionContent;
use lex_parser::lex::ast::{Definition, Session, TextContent};

fn main() {
    let subject = TextContent::from_string("Term".to_string(), None);
    let session = Session::with_title("Nested".to_string());
    let _definition = Definition::new(subject, vec![SessionContent::Session(session)]);
}
