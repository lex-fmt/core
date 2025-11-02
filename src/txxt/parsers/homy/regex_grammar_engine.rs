//! Linebased Regex Grammar Engine
//!
//! A generic, standalone regex-based pattern matching engine for token sequences.
//! This engine is completely decoupled from txxt - it works with any token type names.
//!
//! ## Design
//!
//! The engine converts a sequence of token type names into a space-separated string,
//! then matches that string against regex patterns. This allows using standard regex
//! for grammar rules without building a custom parser.
//!
//! ## Example
//!
//! ```text
//! Tokens:   ["SUBJECT", "INDENT", "PARAGRAPH", "DEDENT"]
//! String:   "SUBJECT INDENT PARAGRAPH DEDENT"
//! Pattern:  "SUBJECT\\s+INDENT\\s+(PARAGRAPH|LIST)\\s+DEDENT"
//! Result:   Match with capture groups
//! ```
//!
//! ## Important
//!
//! This module is completely generic. No txxt knowledge here.
//! It's tested independently with simple token names like "A", "B", "C".

use regex::Regex;
use std::fmt;

/// A sequence of token type names (as strings).
///
/// This is generic - the strings can be anything: "A", "TOKEN_TYPE", "SUBJECT_LINE", etc.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSeq {
    /// List of token type names
    pub tokens: Vec<String>,
}

impl TokenSeq {
    /// Create a new token sequence from a vec of token names
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens }
    }

    /// Convert tokens to a space-separated string for regex matching
    ///
    /// Example: `["A", "B", "C"]` becomes `"A B C"`
    pub fn as_string(&self) -> String {
        self.tokens.join(" ")
    }

    /// Get the token at a given index
    pub fn get(&self, index: usize) -> Option<&str> {
        self.tokens.get(index).map(|s| s.as_str())
    }

    /// Get the number of tokens
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Result of matching a pattern against a token sequence
#[derive(Debug, Clone, PartialEq)]
pub struct RegexMatch {
    /// Whether the pattern matched
    pub matched: bool,

    /// Byte range of the full match in the string representation
    pub full_match: Option<(usize, usize)>,

    /// Byte ranges for each capture group (in order)
    /// If no capture groups, this will be empty
    pub captures: Vec<(usize, usize)>,
}

impl RegexMatch {
    /// Create a successful match with capture ranges
    pub fn matched_with_captures(
        full_match: (usize, usize),
        captures: Vec<(usize, usize)>,
    ) -> Self {
        Self {
            matched: true,
            full_match: Some(full_match),
            captures,
        }
    }

    /// Create a failed match
    pub fn no_match() -> Self {
        Self {
            matched: false,
            full_match: None,
            captures: vec![],
        }
    }
}

/// Error type for regex grammar operations
#[derive(Debug, Clone, PartialEq)]
pub enum RegexError {
    /// Invalid regex pattern
    InvalidPattern(String),
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexError::InvalidPattern(msg) => write!(f, "Invalid regex pattern: {}", msg),
        }
    }
}

impl std::error::Error for RegexError {}

/// A regex-based grammar pattern matcher for token sequences
///
/// Matches token sequences (as strings of space-separated token names) against regex patterns.
pub struct RegexGrammarMatcher {
    pattern: String,
    regex: Regex,
}

impl RegexGrammarMatcher {
    /// Create a new matcher from a regex pattern
    ///
    /// The pattern should match against a space-separated token string.
    ///
    /// # Example
    /// ```ignore
    /// let matcher = RegexGrammarMatcher::new("A\\s+B")?;
    /// ```
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        let regex = Regex::new(pattern).map_err(|e| RegexError::InvalidPattern(e.to_string()))?;

        Ok(Self {
            pattern: pattern.to_string(),
            regex,
        })
    }

    /// Match a pattern against a token sequence
    ///
    /// Returns information about the match including capture groups
    pub fn match_tokens(&self, seq: &TokenSeq) -> RegexMatch {
        let seq_string = seq.as_string();

        match self.regex.captures(&seq_string) {
            Some(caps) => {
                // Get the full match range
                let full_match = caps.get(0).map(|m| (m.start(), m.end()));

                // Get all capture groups (excluding group 0, which is the full match)
                let captures = (1..caps.len())
                    .filter_map(|i| caps.get(i).map(|m| (m.start(), m.end())))
                    .collect();

                RegexMatch::matched_with_captures(full_match.unwrap(), captures)
            }
            None => RegexMatch::no_match(),
        }
    }

    /// Get the tokens from a specific capture group
    ///
    /// # Arguments
    /// * `seq` - The token sequence that was matched
    /// * `capture_idx` - The capture group index (0-based, where 0 is the first capture group)
    ///
    /// # Returns
    /// The token names that were captured in this group
    pub fn get_capture(
        &self,
        seq: &TokenSeq,
        capture_idx: usize,
    ) -> Result<Vec<String>, RegexError> {
        if capture_idx >= seq.len() {
            return Ok(vec![]);
        }

        // Get the string representation
        let seq_string = seq.as_string();

        // Try to match and get the capture
        match self.regex.captures(&seq_string) {
            Some(caps) => {
                // Capture groups are 1-indexed in the regex (0 is full match)
                // So we add 1 to convert from our 0-indexed capture_idx
                if let Some(cap) = caps.get(capture_idx + 1) {
                    let cap_str = cap.as_str();
                    // Split the captured string back into tokens
                    let tokens: Vec<String> =
                        cap_str.split_whitespace().map(|s| s.to_string()).collect();
                    Ok(tokens)
                } else {
                    Ok(vec![])
                }
            }
            None => Ok(vec![]),
        }
    }

    /// Get the pattern string
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_single_token_match() {
        let seq = TokenSeq::new(vec!["A".to_string()]);
        let matcher = RegexGrammarMatcher::new("A").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_simple_single_token_no_match() {
        let seq = TokenSeq::new(vec!["A".to_string()]);
        let matcher = RegexGrammarMatcher::new("B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(!result.matched);
    }

    #[test]
    fn test_sequence_match() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("A\\s+B\\s+C").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_sequence_partial_match() {
        // Pattern matches but only part of the sequence
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("A\\s+B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_sequence_no_match() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("A\\s+C").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(!result.matched);
    }

    #[test]
    fn test_capture_groups() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A)\\s+(B)\\s+(C)").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
        assert_eq!(result.captures.len(), 3); // Three capture groups
    }

    #[test]
    fn test_capture_groups_extraction() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A)\\s+(B)\\s+(C)").unwrap();

        let cap0 = matcher.get_capture(&seq, 0).unwrap();
        assert_eq!(cap0, vec!["A"]);

        let cap1 = matcher.get_capture(&seq, 1).unwrap();
        assert_eq!(cap1, vec!["B"]);

        let cap2 = matcher.get_capture(&seq, 2).unwrap();
        assert_eq!(cap2, vec!["C"]);
    }

    #[test]
    fn test_optional_elements() {
        let seq = TokenSeq::new(vec!["A".to_string(), "C".to_string()]);
        // Pattern needs to handle the space - if B is optional, the space might not match
        let matcher = RegexGrammarMatcher::new("A\\s+(B\\s+)?C").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_optional_elements_present() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("A\\s+(B)?\\s+C").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_alternatives() {
        let seq = TokenSeq::new(vec!["A".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A|B|C)").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_alternatives_second() {
        let seq = TokenSeq::new(vec!["B".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A|B|C)").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_alternatives_no_match() {
        let seq = TokenSeq::new(vec!["D".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A|B|C)").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(!result.matched);
    }

    #[test]
    fn test_quantifiers_plus() {
        let seq = TokenSeq::new(vec!["A".to_string(), "A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("A+\\s+B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_quantifiers_plus_single() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("A+\\s+B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_quantifiers_star() {
        let seq = TokenSeq::new(vec!["B".to_string()]);
        // A* matches zero or more A's, so this should match just B
        let matcher = RegexGrammarMatcher::new("(A\\s+)*B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_quantifiers_star_multiple() {
        let seq = TokenSeq::new(vec!["A".to_string(), "A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("A*\\s+B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_no_match() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("X\\s+Y").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(!result.matched);
    }

    #[test]
    fn test_invalid_pattern() {
        // Unmatched parenthesis
        let result = RegexGrammarMatcher::new("(A");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_pattern() {
        let seq = TokenSeq::new(vec![
            "START".to_string(),
            "MIDDLE".to_string(),
            "MIDDLE".to_string(),
            "END".to_string(),
        ]);
        let matcher = RegexGrammarMatcher::new("START\\s+(MIDDLE\\s+)+END").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_token_seq_as_string() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        assert_eq!(seq.as_string(), "A B C");
    }

    #[test]
    fn test_token_seq_empty() {
        let seq = TokenSeq::new(vec![]);
        assert!(seq.is_empty());
        assert_eq!(seq.len(), 0);
    }

    #[test]
    fn test_token_seq_get() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        assert_eq!(seq.get(0), Some("A"));
        assert_eq!(seq.get(1), Some("B"));
        assert_eq!(seq.get(2), None);
    }

    #[test]
    fn test_multiple_capture_groups_with_alternatives() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A|X)\\s+(B|Y)\\s+(C|Z)").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
        assert_eq!(result.captures.len(), 3);
    }

    #[test]
    fn test_pattern_with_multiple_spaces() {
        // The regex should handle variable spacing between tokens in the pattern
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        // \\s+ means one or more whitespace characters
        let matcher = RegexGrammarMatcher::new("A\\s+B").unwrap();
        let result = matcher.match_tokens(&seq);
        assert!(result.matched);
    }

    #[test]
    fn test_get_capture_out_of_bounds() {
        let seq = TokenSeq::new(vec!["A".to_string(), "B".to_string()]);
        let matcher = RegexGrammarMatcher::new("(A)\\s+(B)").unwrap();
        let result: Result<Vec<String>, _> = matcher.get_capture(&seq, 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![] as Vec<String>); // Out of bounds returns empty
    }

    #[test]
    fn test_multitoken_capture() {
        let seq = TokenSeq::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ]);
        let matcher = RegexGrammarMatcher::new("(A\\s+B)\\s+(C\\s+D)").unwrap();

        let cap0 = matcher.get_capture(&seq, 0).unwrap();
        assert_eq!(cap0, vec!["A", "B"]);

        let cap1 = matcher.get_capture(&seq, 1).unwrap();
        assert_eq!(cap1, vec!["C", "D"]);
    }
}
