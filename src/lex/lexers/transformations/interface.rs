//! Transformation interface for token stream transformations
//!
//! This module defines the core `Transformation` trait that all token transformations implement.
//! A transformation is a pure function that takes a token stream and produces a transformed token stream.
//!
//! Design principles:
//! - Transformations are pure: same input always produces same output
//! - Transformations operate on token streams: Vec<(Token, Range<usize>)> -> Vec<(Token, Range<usize>)>
//! - Transformations have metadata: name and description for documentation/debugging
//! - Transformations are composable: can be chained in pipelines

use crate::lex::lexers::Token;

/// A transformation that processes a token stream
///
/// Transformations are the building blocks of lexer pipelines. Each transformation
/// takes a stream of tokens with their source locations and produces a transformed
/// stream with updated tokens and locations.
///
/// # Examples
///
/// ```ignore
/// struct MyTransformation;
///
/// impl Transformation for MyTransformation {
///     fn name(&self) -> &str {
///         "my_transformation"
///     }
///
///     fn description(&self) -> &str {
///         "Does something to tokens"
///     }
///
///     fn transform(&self, tokens: Vec<(Token, Range<usize>)>) -> Vec<(Token, Range<usize>)> {
///         // Transform the tokens
///         tokens
///     }
/// }
/// ```
pub trait Transformation {
    /// Returns the name of this transformation
    ///
    /// Names should be lowercase with underscores (e.g., "normalize_whitespace")
    fn name(&self) -> &str;

    /// Returns a human-readable description of what this transformation does
    fn description(&self) -> &str;

    /// Apply this transformation to a token stream
    ///
    /// # Arguments
    /// * `tokens` - The input token stream with source locations
    ///
    /// # Returns
    /// The transformed token stream with updated tokens and locations
    fn transform(
        &self,
        tokens: Vec<(Token, std::ops::Range<usize>)>,
    ) -> Vec<(Token, std::ops::Range<usize>)>;
}
