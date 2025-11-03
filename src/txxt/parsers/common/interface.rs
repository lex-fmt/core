//! Common parser interfaces and registry
//!
//! This module defines the `Parser` trait and `ParserRegistry` for pluggable
//! parser implementations. Each parser design (reference, linebased, etc.)
//! implements the `Parser` trait, and the registry allows selecting which
//! parser to use at runtime.

use std::fmt;

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ParserNotFound(String),
    ParsingFailed(String),
    InvalidInput(String),
    IncompatibleInput(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ParserNotFound(name) => write!(f, "Parser '{}' not found", name),
            ParseError::ParsingFailed(msg) => write!(f, "Parsing failed: {}", msg),
            ParseError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ParseError::IncompatibleInput(msg) => write!(f, "Incompatible input format: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Input to the parser - can be tokens or a token tree depending on parser requirements
#[derive(Debug, Clone)]
pub enum ParserInput {
    /// Standard token stream with source locations (for reference parser)
    Tokens(Vec<(crate::txxt::lexers::Token, std::ops::Range<usize>)>),
    /// Line-based token trees (for linebased parser)
    LineTokenTrees(Vec<crate::txxt::lexers::LineTokenTree>),
}

/// Trait for pluggable parser implementations
pub trait Parser: Send + Sync {
    /// Return the name of this parser implementation
    fn name(&self) -> &'static str;

    /// Parse input into an AST document
    ///
    /// # Arguments
    /// * `input` - The parser input (tokens or tree structure)
    /// * `source` - The original source text (for location tracking)
    ///
    /// # Returns
    /// The parsed Document or parsing error
    fn parse(
        &self,
        input: ParserInput,
        source: &str,
    ) -> Result<crate::txxt::parsers::Document, ParseError>;

    /// Check if this parser supports a particular input type
    fn supports_input(&self, input: &ParserInput) -> bool;
}

/// Registry for parser implementations
///
/// This registry holds all available parser implementations and provides
/// methods to select and use them.
#[derive(Clone)]
pub struct ParserRegistry {
    parsers: std::collections::HashMap<String, std::sync::Arc<dyn Parser>>,
}

impl ParserRegistry {
    /// Create a new parser registry
    pub fn new() -> Self {
        ParserRegistry {
            parsers: std::collections::HashMap::new(),
        }
    }

    /// Register a parser implementation
    pub fn register(&mut self, parser: std::sync::Arc<dyn Parser>) {
        self.parsers.insert(parser.name().to_string(), parser);
    }

    /// Get a registered parser by name
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn Parser>> {
        self.parsers.get(name).cloned()
    }

    /// Check if a parser is registered
    pub fn has(&self, name: &str) -> bool {
        self.parsers.contains_key(name)
    }

    /// Get list of available parser names
    pub fn available(&self) -> Vec<String> {
        let mut names: Vec<_> = self.parsers.keys().cloned().collect();
        names.sort();
        names
    }

    /// Parse using a registered parser
    pub fn parse(
        &self,
        name: &str,
        input: ParserInput,
        source: &str,
    ) -> Result<crate::txxt::parsers::Document, ParseError> {
        let parser = self
            .get(name)
            .ok_or_else(|| ParseError::ParserNotFound(name.to_string()))?;

        // Check if parser supports this input type
        if !parser.supports_input(&input) {
            return Err(ParseError::IncompatibleInput(format!(
                "Parser '{}' does not support this input format",
                name
            )));
        }

        parser.parse(input, source)
    }

    /// Get the global parser registry
    pub fn global() -> &'static std::sync::Mutex<ParserRegistry> {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<std::sync::Mutex<ParserRegistry>> = OnceLock::new();
        REGISTRY.get_or_init(|| std::sync::Mutex::new(ParserRegistry::new()))
    }

    /// Initialize the global registry with default parsers
    pub fn init_defaults() {
        let mut registry = Self::global().lock().unwrap();
        if registry.available().is_empty() {
            registry.register(std::sync::Arc::new(ReferenceParserImpl));
            registry.register(std::sync::Arc::new(LineBasedParserImpl));
        }
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation wrapper for the reference parser
pub struct ReferenceParserImpl;

impl Parser for ReferenceParserImpl {
    fn name(&self) -> &'static str {
        "reference"
    }

    fn parse(
        &self,
        input: ParserInput,
        source: &str,
    ) -> Result<crate::txxt::parsers::Document, ParseError> {
        match input {
            ParserInput::Tokens(tokens) => {
                // Call the actual reference parser
                crate::txxt::parsers::parse(tokens, source)
                    .map_err(|_| ParseError::ParsingFailed("Reference parser failed".to_string()))
            }
            ParserInput::LineTokenTrees(_) => Err(ParseError::IncompatibleInput(
                "Reference parser requires token stream, not line token trees".to_string(),
            )),
        }
    }

    fn supports_input(&self, input: &ParserInput) -> bool {
        matches!(input, ParserInput::Tokens(_))
    }
}

/// Implementation wrapper for the linebased (linebased) parser
pub struct LineBasedParserImpl;

impl Parser for LineBasedParserImpl {
    fn name(&self) -> &'static str {
        "linebased"
    }

    fn parse(
        &self,
        input: ParserInput,
        source: &str,
    ) -> Result<crate::txxt::parsers::Document, ParseError> {
        match input {
            ParserInput::LineTokenTrees(trees) => {
                // The new declarative grammar parser uses LineContainerToken directly
                // For backward compatibility, convert Vec<LineTokenTree> back to LineContainerToken
                let container = convert_token_trees_to_container(&trees);
                crate::txxt::parsers::linebased::parse_experimental_v2(container, source).map_err(
                    |e| ParseError::ParsingFailed(format!("LineBased parser failed: {}", e)),
                )
            }
            ParserInput::Tokens(_) => Err(ParseError::IncompatibleInput(
                "LineBased parser requires line token trees, not token stream".to_string(),
            )),
        }
    }

    fn supports_input(&self, input: &ParserInput) -> bool {
        matches!(input, ParserInput::LineTokenTrees(_))
    }
}

/// Convert Vec<LineTokenTree> back to LineContainerToken for the new parser
/// This is a backward compatibility bridge while the lexer pipeline transitions
fn convert_token_trees_to_container(
    trees: &[crate::txxt::lexers::LineTokenTree],
) -> crate::txxt::lexers::LineContainerToken {
    use crate::txxt::lexers::{LineContainerToken, LineTokenTree};

    // Convert the tree structure back to LineContainerToken format
    fn convert_tree(tree: &LineTokenTree) -> Vec<LineContainerToken> {
        match tree {
            LineTokenTree::Token(token) => vec![LineContainerToken::Token(token.clone())],
            LineTokenTree::Block(children) => {
                let converted_children = children.iter().flat_map(convert_tree).collect();
                vec![LineContainerToken::Container {
                    children: converted_children,
                }]
            }
            LineTokenTree::Container(_) => {
                // Skip legacy containers (shouldn't happen in new code)
                vec![]
            }
        }
    }

    let children = trees.iter().flat_map(convert_tree).collect();

    // Wrap everything in a root container
    LineContainerToken::Container { children }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_registry_register_and_get() {
        let mut registry = ParserRegistry::new();
        let parser = std::sync::Arc::new(ReferenceParserImpl);

        registry.register(parser.clone());
        assert!(registry.get("reference").is_some());
        assert_eq!(registry.get("reference").unwrap().name(), "reference");
    }

    #[test]
    fn test_parser_registry_has() {
        let mut registry = ParserRegistry::new();
        registry.register(std::sync::Arc::new(ReferenceParserImpl));

        assert!(registry.has("reference"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_parser_registry_available() {
        let mut registry = ParserRegistry::new();
        registry.register(std::sync::Arc::new(ReferenceParserImpl));
        registry.register(std::sync::Arc::new(LineBasedParserImpl));

        let available = registry.available();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&"reference".to_string()));
        assert!(available.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_parser_registry_parse_not_found() {
        let registry = ParserRegistry::new();
        let input = ParserInput::Tokens(vec![]);
        let result = registry.parse("nonexistent", input, "test");

        assert!(result.is_err());
        match result {
            Err(ParseError::ParserNotFound(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected ParserNotFound error"),
        }
    }

    #[test]
    fn test_reference_parser_name() {
        let parser = ReferenceParserImpl;
        assert_eq!(parser.name(), "reference");
    }

    #[test]
    fn test_linebased_parser_name() {
        let parser = LineBasedParserImpl;
        assert_eq!(parser.name(), "linebased");
    }

    #[test]
    fn test_reference_parser_supports_tokens() {
        let parser = ReferenceParserImpl;
        let input = ParserInput::Tokens(vec![]);

        assert!(parser.supports_input(&input));
    }

    #[test]
    fn test_reference_parser_does_not_support_tree() {
        let parser = ReferenceParserImpl;
        let tree = crate::txxt::lexers::LineTokenTree::Block(vec![]);
        let input = ParserInput::LineTokenTrees(vec![tree]);

        assert!(!parser.supports_input(&input));
    }

    #[test]
    fn test_linebased_parser_supports_tree() {
        let parser = LineBasedParserImpl;
        let tree = crate::txxt::lexers::LineTokenTree::Block(vec![]);
        let input = ParserInput::LineTokenTrees(vec![tree]);

        assert!(parser.supports_input(&input));
    }

    #[test]
    fn test_linebased_parser_does_not_support_tokens() {
        let parser = LineBasedParserImpl;
        let input = ParserInput::Tokens(vec![]);

        assert!(!parser.supports_input(&input));
    }

    #[test]
    fn test_registry_parse_incompatible_input() {
        let mut registry = ParserRegistry::new();
        registry.register(std::sync::Arc::new(ReferenceParserImpl));

        // Try to parse with reference parser and an incompatible input (line token trees)
        let tree = crate::txxt::lexers::LineTokenTree::Block(vec![]);
        let input = ParserInput::LineTokenTrees(vec![tree]);
        let result = registry.parse("reference", input, "test");

        assert!(result.is_err());
        match result {
            Err(ParseError::IncompatibleInput(_)) => {
                // Success
            }
            _ => panic!("Expected IncompatibleInput error"),
        }
    }
}
