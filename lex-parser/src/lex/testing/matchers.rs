//! Text matching utilities for AST assertions

/// Text matching strategies for assertions
#[derive(Debug, Clone)]
pub enum TextMatch {
    /// Exact text match
    Exact(String),
    /// Text starts with prefix
    StartsWith(String),
    /// Text contains substring
    Contains(String),
}

impl TextMatch {
    /// Check if the actual text matches this pattern (returns bool)
    pub fn matches(&self, actual: &str) -> bool {
        match self {
            TextMatch::Exact(expected) => actual == expected,
            TextMatch::StartsWith(prefix) => actual.starts_with(prefix),
            TextMatch::Contains(substring) => actual.contains(substring),
        }
    }

    /// Assert that the actual text matches this pattern
    pub fn assert(&self, actual: &str, context: &str) {
        match self {
            TextMatch::Exact(expected) => {
                assert_eq!(
                    actual, expected,
                    "{}: Expected text to be '{}', but got '{}'",
                    context, expected, actual
                );
            }
            TextMatch::StartsWith(prefix) => {
                assert!(
                    actual.starts_with(prefix),
                    "{}: Expected text to start with '{}', but got '{}'",
                    context,
                    prefix,
                    actual
                );
            }
            TextMatch::Contains(substring) => {
                assert!(
                    actual.contains(substring),
                    "{}: Expected text to contain '{}', but got '{}'",
                    context,
                    substring,
                    actual
                );
            }
        }
    }
}
