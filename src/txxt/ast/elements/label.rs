//! Label element definition

use super::super::location::Location;
use std::fmt;

/// A label represents a named identifier in txxt documents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub value: String,
    pub location: Option<Location>,
}

impl Label {
    pub fn new(value: String) -> Self {
        Self {
            value,
            location: None,
        }
    }
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
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
        let label = Label::new("test".to_string()).with_location(Some(location));
        assert_eq!(label.location, Some(location));
    }
}
