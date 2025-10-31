//! Parameter element
//!
//! A parameter is a key with an optional value, attached to
//! annotations and foreign blocks to convey structured metadata.
//!
//! Business use:
//! - Lightweight configuration and flags (boolean or key=value)
//! - Enables filtering, routing, and automation in tools
//!
//! Examples:
//! - `priority=high`
//! - `draft`
//!
//! Learn More:
//! - Parameters spec: docs/specs/v1/elements/parameters.txxt

use super::super::location::Location;
use std::fmt;

/// A parameter represents a key-value pair, with optional value
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: Option<String>,
    pub location: Option<Location>,
}

impl Parameter {
    pub fn new(key: String, value: Option<String>) -> Self {
        Self {
            key,
            value,
            location: None,
        }
    }
    pub fn boolean(key: String) -> Self {
        Self {
            key,
            value: None,
            location: None,
        }
    }
    pub fn with_value(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
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
    fn test_parameter_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let param = Parameter::new("key".to_string(), Some("value".to_string()))
            .with_location(Some(location));
        assert_eq!(param.location, Some(location));
    }
}
