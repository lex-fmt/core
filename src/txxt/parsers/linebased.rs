//! Grammar Engine Parser Module
//!
//! This parser design started as a way to parse based on a declarative grammar alone.
//! For this reason, the tokenizer transformed simpler into line based tokens, which makes the
//! matching easier.
//!
//! However, mid development, we realize the wall: the regex approach cannot count and keep tabs
//! of indent and dedent levels.
//!
//! Hence, we adapted this parser to receive a token tree, which makes the matching easer, but that
//! does require us to walk the tree and manage the recursion ourselves.
//! The pattern matcher grammar part will only look at the current level's tokens, and resort to a
//! more complex and traditional code based approach to handle the recursion.
//!
//!
//! This module implements a multi-pass parsing approach that separates concerns:
//! - Tree Walking (orchestration and recursion handling)
//! - Regex Grammar Engine (generic pattern matching, no txxt knowledge)
//! - Pattern Matching (grammar recognition using the regex engine)
//! - AST Construction (converting patterns to nodes)
//!
//! Design
//!
//! The parser operates in phases:
//! 1. Receive a LineTokenTree from the linebased lexer
//! 2. Walk the tree recursively, flattening tokens at each level
//! 3. Use the regex grammar engine to match patterns against token sequences
//! 4. Convert matched patterns to AST nodes via the unwrapper
//! 5. Return final Document
//!
//! This separation makes each component testable independently.

pub mod declarative_grammar;
pub mod engine;
pub mod regex_grammar_engine;
pub mod txxt_grammar;
pub mod unwrapper;

pub use engine::parse_experimental_v2;
pub use regex_grammar_engine::{RegexGrammarMatcher, TokenSeq};
pub use txxt_grammar::{MatchedPattern, TxxtGrammarRules};
