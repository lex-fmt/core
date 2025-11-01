//! Label element
//!
//! A label is a short identifier used by annotations and other
//! elements. Labels server similar roles but have relevant differences from:
//! - Tags: An annotation can only have one label, while tags are typically multiple.
//! - IDS: labels are not unique, even in the the same element
//!
//! Labels support dot notation for namespaces:
//! Namespaced: txxt.internal, plugin.myapp.custom
//! Namespaces are user defined, with the exception of the doc and txxt namespaces which are reserved.
//!
//! Syntax:
//! <letter> (<letter> | <digit> | "_" | "-" | ".")*
//!
//!
//! Learn More:
//! - Labels spec: docs/specs/v1/elements/labels.txxt

use super::super::location::{Location, Position};
use std::fmt;

/// A label represents a named identifier in txxt documents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub value: String,
    pub location: Location,
}

impl Label {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(value: String) -> Self {
        Self {
            value,
            location: Self::default_location(),
        }
    }
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
            location: Self::default_location(),
        }
    }
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
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
    fn test_label_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let label = Label::new("test".to_string()).with_location(location);
        assert_eq!(label.location, location);
    }
}
