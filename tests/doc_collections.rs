use lex::lex::pipeline::Parser;
use lex::lex::testing::lexplore::Lexplore;
use rstest::rstest;

fn assert_not_empty(doc: &lex::lex::ast::Document, label: &str, parser: Parser) {
    assert!(
        !doc.root.children.is_empty(),
        "{} should have content for {:?}",
        label,
        parser
    );
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_000_paragraphs(parser: Parser) {
    let doc = Lexplore::trifecta(0).parse_with(parser);
    assert_not_empty(&doc, "trifecta_000_paragraphs", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_010_paragraphs_sessions_flat_single(parser: Parser) {
    let doc = Lexplore::trifecta(10).parse_with(parser);
    assert_not_empty(&doc, "trifecta_010_paragraphs_sessions_flat_single", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_020_paragraphs_sessions_flat_multiple(parser: Parser) {
    let doc = Lexplore::trifecta(20).parse_with(parser);
    assert_not_empty(
        &doc,
        "trifecta_020_paragraphs_sessions_flat_multiple",
        parser,
    );
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_030_paragraphs_sessions_nested_multiple(parser: Parser) {
    let doc = Lexplore::trifecta(30).parse_with(parser);
    assert_not_empty(
        &doc,
        "trifecta_030_paragraphs_sessions_nested_multiple",
        parser,
    );
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_040_lists(parser: Parser) {
    let doc = Lexplore::trifecta(40).parse_with(parser);
    assert_not_empty(&doc, "trifecta_040_lists", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_050_paragraph_lists(parser: Parser) {
    let doc = Lexplore::trifecta(50).parse_with(parser);
    assert_not_empty(&doc, "trifecta_050_paragraph_lists", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_060_trifecta_nesting(parser: Parser) {
    let doc = Lexplore::trifecta(60).parse_with(parser);
    assert_not_empty(&doc, "trifecta_060_trifecta_nesting", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn trifecta_070_trifecta_flat_simple(parser: Parser) {
    let doc = Lexplore::trifecta(70).parse_with(parser);
    assert_not_empty(&doc, "trifecta_070_trifecta_flat_simple", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn benchmark_010_kitchensink(parser: Parser) {
    let doc = Lexplore::benchmark(10).parse_with(parser);
    assert_not_empty(&doc, "benchmark_010_kitchensink", parser);
}
