use lex::lex::pipeline::Parser;
use lex::lex::testing::lexplore::Lexplore;
use rstest::rstest;

fn assert_non_empty(doc: &lex::lex::ast::Document, label: &str, parser: Parser) {
    assert!(
        !doc.root.children.is_empty(),
        "{} should produce items for {:?}",
        label,
        parser
    );
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_01(parser: Parser) {
    let doc = Lexplore::verbatim(1).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_01", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_02(parser: Parser) {
    let doc = Lexplore::verbatim(2).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_02", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_03(parser: Parser) {
    let doc = Lexplore::verbatim(3).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_03", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_04(parser: Parser) {
    let doc = Lexplore::verbatim(4).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_04", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_05(parser: Parser) {
    let doc = Lexplore::verbatim(5).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_05", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_06(parser: Parser) {
    let doc = Lexplore::verbatim(6).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_06", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_07(parser: Parser) {
    let doc = Lexplore::verbatim(7).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_07", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_08(parser: Parser) {
    let doc = Lexplore::verbatim(8).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_08", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_09(parser: Parser) {
    let doc = Lexplore::verbatim(9).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_09", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_10(parser: Parser) {
    let doc = Lexplore::verbatim(10).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_10", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_11(parser: Parser) {
    let doc = Lexplore::verbatim(11).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_11", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_12(parser: Parser) {
    let doc = Lexplore::verbatim(12).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_12", parser);
}

#[rstest(parser => [Parser::Reference, Parser::Linebased])]
fn verbatim_dual_13(parser: Parser) {
    let doc = Lexplore::verbatim(13).parse_with(parser);
    assert_non_empty(&doc, "verbatim_dual_13", parser);
}
