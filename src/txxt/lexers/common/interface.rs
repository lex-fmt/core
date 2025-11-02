//! Common lexer interfaces and registry
//!
//! This module defines the `Lexer` trait and `LexerRegistry` for pluggable
//! lexer implementations. Each lexer design (indentation, linebased, etc.)
//! implements the `Lexer` trait, and the registry allows selecting which
//! lexer to use at runtime.

use std::fmt;

/// Errors that can occur during lexing
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    LexerNotFound(String),
    TokenizationFailed(String),
    InvalidInput(String),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::LexerNotFound(name) => write!(f, "Lexer '{}' not found", name),
            LexError::TokenizationFailed(msg) => write!(f, "Tokenization failed: {}", msg),
            LexError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for LexError {}

/// Output of the lexer - can be tokens or a token tree depending on implementation
#[derive(Debug, Clone)]
pub enum LexerOutput {
    /// Standard token stream with source locations
    Tokens(Vec<(crate::txxt::lexers::Token, std::ops::Range<usize>)>),
    /// Line-based token tree (from linebased lexer)
    LineTokenTrees(Vec<crate::txxt::lexers::LineTokenTree>),
}

/// Trait for pluggable lexer implementations
pub trait Lexer: Send + Sync {
    /// Return the name of this lexer implementation
    fn name(&self) -> &'static str;

    /// Tokenize source text into tokens
    ///
    /// # Arguments
    /// * `source` - The source text to tokenize
    ///
    /// # Returns
    /// The lexer output (tokens or tree structure depending on implementation)
    fn tokenize(&self, source: &str) -> Result<LexerOutput, LexError>;
}

/// Registry for lexer implementations
///
/// This registry holds all available lexer implementations and provides
/// methods to select and use them.
#[derive(Clone)]
pub struct LexerRegistry {
    lexers: std::collections::HashMap<String, std::sync::Arc<dyn Lexer>>,
}

impl LexerRegistry {
    /// Create a new lexer registry
    pub fn new() -> Self {
        LexerRegistry {
            lexers: std::collections::HashMap::new(),
        }
    }

    /// Register a lexer implementation
    pub fn register(&mut self, lexer: std::sync::Arc<dyn Lexer>) {
        self.lexers.insert(lexer.name().to_string(), lexer);
    }

    /// Get a registered lexer by name
    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn Lexer>> {
        self.lexers.get(name).cloned()
    }

    /// Check if a lexer is registered
    pub fn has(&self, name: &str) -> bool {
        self.lexers.contains_key(name)
    }

    /// Get list of available lexer names
    pub fn available(&self) -> Vec<String> {
        let mut names: Vec<_> = self.lexers.keys().cloned().collect();
        names.sort();
        names
    }

    /// Tokenize using a registered lexer
    pub fn tokenize(&self, name: &str, source: &str) -> Result<LexerOutput, LexError> {
        let lexer = self
            .get(name)
            .ok_or_else(|| LexError::LexerNotFound(name.to_string()))?;
        lexer.tokenize(source)
    }

    /// Get the global lexer registry
    pub fn global() -> &'static std::sync::Mutex<LexerRegistry> {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<std::sync::Mutex<LexerRegistry>> = OnceLock::new();
        REGISTRY.get_or_init(|| std::sync::Mutex::new(LexerRegistry::new()))
    }

    /// Initialize the global registry with default lexers
    pub fn init_defaults() {
        let mut registry = Self::global().lock().unwrap();
        if registry.available().is_empty() {
            registry.register(std::sync::Arc::new(IndentationLexerImpl));
            registry.register(std::sync::Arc::new(LinebasedLexerImpl));
        }
    }
}

impl Default for LexerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation wrapper for the indentation lexer
pub struct IndentationLexerImpl;

impl Lexer for IndentationLexerImpl {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn tokenize(&self, source: &str) -> Result<LexerOutput, LexError> {
        // Call the actual indentation lexer
        let output = crate::txxt::lexers::lex(source);
        Ok(LexerOutput::Tokens(output))
    }
}

/// Implementation wrapper for the linebased (linebased) lexer
pub struct LinebasedLexerImpl;

impl Lexer for LinebasedLexerImpl {
    fn name(&self) -> &'static str {
        "linebased"
    }

    fn tokenize(&self, source: &str) -> Result<LexerOutput, LexError> {
        // Call the actual linebased lexer
        let trees = crate::txxt::lexers::_lex(source)
            .map_err(|e| LexError::TokenizationFailed(format!("{:?}", e)))?;
        Ok(LexerOutput::LineTokenTrees(trees))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_registry_register_and_get() {
        let mut registry = LexerRegistry::new();
        let lexer = std::sync::Arc::new(IndentationLexerImpl);

        registry.register(lexer.clone());
        assert!(registry.get("indentation").is_some());
        assert_eq!(registry.get("indentation").unwrap().name(), "indentation");
    }

    #[test]
    fn test_lexer_registry_has() {
        let mut registry = LexerRegistry::new();
        registry.register(std::sync::Arc::new(IndentationLexerImpl));

        assert!(registry.has("indentation"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_lexer_registry_available() {
        let mut registry = LexerRegistry::new();
        registry.register(std::sync::Arc::new(IndentationLexerImpl));
        registry.register(std::sync::Arc::new(LinebasedLexerImpl));

        let available = registry.available();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&"indentation".to_string()));
        assert!(available.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_lexer_registry_tokenize_not_found() {
        let registry = LexerRegistry::new();
        let result = registry.tokenize("nonexistent", "test");

        assert!(result.is_err());
        match result {
            Err(LexError::LexerNotFound(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected LexerNotFound error"),
        }
    }

    #[test]
    fn test_indentation_lexer_name() {
        let lexer = IndentationLexerImpl;
        assert_eq!(lexer.name(), "indentation");
    }

    #[test]
    fn test_linebased_lexer_name() {
        let lexer = LinebasedLexerImpl;
        assert_eq!(lexer.name(), "linebased");
    }

    #[test]
    fn test_indentation_lexer_tokenize_simple() {
        let lexer = IndentationLexerImpl;
        let result = lexer.tokenize("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            LexerOutput::Tokens(_) => {
                // Success - indentation lexer produces tokens
            }
            _ => panic!("Expected Tokens output from indentation lexer"),
        }
    }

    #[test]
    fn test_linebased_lexer_tokenize_simple() {
        let lexer = LinebasedLexerImpl;
        let result = lexer.tokenize("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            LexerOutput::LineTokenTrees(_) => {
                // Success - linebased lexer produces tree
            }
            _ => panic!("Expected LineTokenTrees output from linebased lexer"),
        }
    }

    #[test]
    fn test_registry_tokenize_indentation() {
        let mut registry = LexerRegistry::new();
        registry.register(std::sync::Arc::new(IndentationLexerImpl));

        let result = registry.tokenize("indentation", "hello");
        assert!(result.is_ok());

        match result.unwrap() {
            LexerOutput::Tokens(_) => {
                // Success
            }
            _ => panic!("Expected Tokens from indentation lexer"),
        }
    }

    #[test]
    fn test_registry_tokenize_linebased() {
        let mut registry = LexerRegistry::new();
        registry.register(std::sync::Arc::new(LinebasedLexerImpl));

        let result = registry.tokenize("linebased", "hello");
        assert!(result.is_ok());

        match result.unwrap() {
            LexerOutput::LineTokenTrees(_) => {
                // Success
            }
            _ => panic!("Expected LineTokenTrees from linebased lexer"),
        }
    }
}
