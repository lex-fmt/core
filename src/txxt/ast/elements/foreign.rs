//! Foreign block element
//!
//! A foreign block embeds content that is not txxt formated.
//! Typically this can either be binary data, such as images, or text in some formal langauge
//! like a programming language excerpt in Python.
//!
//! Note that a foreign block can forgo content all together (i.e. binaries won't encode conetent).
//!
//! The subject introduces what the content is, and the closing annotation terminates the block.
//! The annotation can take any valid annotation form, and the label is user defined. As a convention
//! tough,  if the content is to be interpreted by a tool, the label should be the name of the tool/language.
//! While the txxt software will not parse the content, it will preserve it exactly as it is, and can be used
//! to format the content in editors and other tools.
//!
//! Syntax:
//! <subject-line>
//! <indent> <content> ... any number of content elements
//! <dedent>  <annotation>
//!
//! Examples:
//!     Images:
//!         Sunset Photo:
//!         :: image type=jpg, src=sunset.jpg :: As the sun sets over the ocean.
//!     Code:
//!         JavaScript Example:
//!             function hello() {
//!                 return "world";
//!             }
//!      :: javascript ::
//!
//! Learn More:
//! - Foreign blocks spec: docs/specs/v1/elements/foreign.txxt
//!

use super::super::location::{Location, Position};
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::super::traits::Visitor;
use super::annotation::Annotation;
use std::fmt;

/// A foreign block represents content from another format/system
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    pub subject: TextContent,
    pub content: TextContent,
    pub closing_annotation: Annotation,
    pub location: Location,
}

impl ForeignBlock {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(content, None),
            closing_annotation,
            location: Self::default_location(),
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            content: TextContent::from_string(String::new(), None),
            closing_annotation,
            location: Self::default_location(),
        }
    }
    #[deprecated(note = "Use at(location) instead")]
    pub fn with_location(self, location: Location) -> Self {
        self.at(location)
    }
    /// Preferred builder
    pub fn at(mut self, location: Location) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for ForeignBlock {
    fn node_type(&self) -> &'static str {
        "ForeignBlock"
    }
    fn display_label(&self) -> String {
        let subject_text = self.subject.as_string();
        if subject_text.len() > 50 {
            format!("{}...", &subject_text[..50])
        } else {
            subject_text.to_string()
        }
    }
    fn location(&self) -> Location {
        self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_foreign_block(self);
        // ForeignBlock has no children to visit - content is opaque
    }
}

impl fmt::Display for ForeignBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ForeignBlock('{}', {} chars, closing: {})",
            self.subject.as_string(),
            self.content.as_string().len(),
            self.closing_annotation.label.value
        )
    }
}
