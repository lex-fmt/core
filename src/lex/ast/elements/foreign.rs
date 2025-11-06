//! Verbatim block element
//!
//! A foreign block embeds content that is not lex formatted.
//! Typically this can either be binary data, such as images, or text in some formal language
//! like a programming language excerpt in Python.
//!
//! Note that a foreign block can forgo content all together (i.e. binaries won't encode content).
//!
//! Structure:
//! - subject: The lead item identifying what the foreign block contains
//! - children: VerbatimLine nodes containing the actual content (can be empty)
//! - closing_annotation: The closing marker (format: `::label::`)
//!
//! The subject introduces what the content is, and the closing annotation terminates the block.
//! The annotation can take any valid annotation form, and the label is user defined. As a convention
//! though, if the content is to be interpreted by a tool, the label should be the name of the tool/language.
//! While the lex software will not parse the content, it will preserve it exactly as it is, and can be used
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
//! - Verbatim blocks spec: docs/specs/v1/elements/foreign.lex
//!

use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::container::VerbatimContainer;
use super::content_item::ContentItem;
use std::fmt;

/// A foreign block represents content from another format/system
#[derive(Debug, Clone, PartialEq)]
pub struct Verbatim {
    pub subject: TextContent,
    pub children: VerbatimContainer,
    pub closing_annotation: Annotation,
    pub location: Range,
}

impl Verbatim {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }

    pub fn new(
        subject: TextContent,
        children: Vec<ContentItem>,
        closing_annotation: Annotation,
    ) -> Self {
        Self {
            subject,
            children: VerbatimContainer::new(children),
            closing_annotation,
            location: Self::default_location(),
        }
    }

    pub fn with_subject(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
        }
    }

    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for Verbatim {
    fn node_type(&self) -> &'static str {
        "VerbatimBlock"
    }
    fn display_label(&self) -> String {
        let subject_text = self.subject.as_string();
        if subject_text.len() > 50 {
            format!("{}...", &subject_text[..50])
        } else {
            subject_text.to_string()
        }
    }
    fn range(&self) -> &Range {
        &self.location
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_verbatim_block(self);
        super::super::traits::visit_children(visitor, &self.children);
    }
}

impl Container for Verbatim {
    fn label(&self) -> &str {
        self.subject.as_string()
    }

    fn children(&self) -> &[ContentItem] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.children
    }
}

impl fmt::Display for Verbatim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VerbatimBlock('{}', {} lines, closing: {})",
            self.subject.as_string(),
            self.children.len(),
            self.closing_annotation.label.value
        )
    }
}
