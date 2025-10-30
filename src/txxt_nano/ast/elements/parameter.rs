//! Parameter element definition

use super::super::span::Location;
use std::fmt;

/// A parameter represents a key-value pair, with optional value
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: Option<String>,
    pub span: Option<Location>,
}

impl Parameter {
    pub fn new(key: String, value: Option<String>) -> Self {
        Self {
            key,
            value,
            span: None,
        }
    }
    pub fn boolean(key: String) -> Self {
        Self {
            key,
            value: None,
            span: None,
        }
    }
    pub fn with_value(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Location>) -> Self {
        self.span = span;
        self
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(v) => write!(f, "{}={}", self.key, v),
            None => write!(f, "{}", self.key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_with_span() {
        let span = super::super::super::span::Location::new(
            super::super::super::span::Position::new(1, 0),
            super::super::super::span::Position::new(1, 10),
        );
        let param =
            Parameter::new("key".to_string(), Some("value".to_string())).with_span(Some(span));
        assert_eq!(param.span, Some(span));
    }
}
