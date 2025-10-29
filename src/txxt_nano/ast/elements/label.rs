//! Label element definition

use super::super::span::Span;
use std::fmt;

/// A label represents a named identifier in txxt documents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub value: String,
    pub span: Option<Span>,
}

impl Label {
    pub fn new(value: String) -> Self {
        Self { value, span: None }
    }
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_with_span() {
        let span = super::super::super::span::Span::new(
            super::super::super::span::Position::new(1, 0),
            super::super::super::span::Position::new(1, 10),
        );
        let label = Label::new("test".to_string()).with_span(Some(span));
        assert_eq!(label.span, Some(span));
    }
}
