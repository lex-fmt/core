//! Integration tests for txxt-nano

use txxt_nano::txxt_nano::Parser;

#[test]
fn test_parser_creation() {
    let _parser = Parser::new();
    // This test verifies that the parser can be created
    // Future tests will verify actual parsing functionality
}

#[test]
fn test_parser_default() {
    let _parser = Parser::default();
    // This test verifies that the parser can be created using Default trait
}
