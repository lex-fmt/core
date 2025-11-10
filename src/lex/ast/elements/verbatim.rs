//! Verbatim block element
//!
//! A verbatim block embeds content that is not lex formatted.
//! Typically this can either be binary data, such as images, or text in some formal language
//! like a programming language excerpt in Python.
//!
//! Note that a verbatim block can forgo content all together (i.e. binaries won't encode content).
//!
//! Structure:
//! - subject: The lead item identifying what the verbatim block contains
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
//! - Verbatim blocks spec: docs/specs/v1/elements/verbatim.lex
//!

use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::container::VerbatimContainer;
use super::content_item::ContentItem;
use std::fmt;
use std::slice;

/// A verbatim block represents content from another format/system.
///
/// # Verbatim Groups
///
/// Verbatim blocks can contain multiple subject/content pairs sharing a single closing
/// annotation. This is useful for sequences like shell command transcripts or grouped
/// code samples in the same language.
///
/// ## Storage Design
///
/// The first subject/content pair is stored directly in the `subject` and `children`
/// fields for backwards compatibility with existing code that expects direct field access.
/// Additional pairs are stored in the private `additional_groups` vector.
///
/// This split storage pattern allows:
/// - Existing code to continue working unchanged (accessing first group via public fields)
/// - New code to iterate over all groups uniformly using the `group()` method
/// - Zero overhead for single-group verbatim blocks (the common case)
///
/// ## Usage
///
/// For single-group blocks, access fields directly:
/// ```ignore
/// let subject = verbatim.subject.as_string();
/// let content = &verbatim.children;
/// ```
///
/// For multi-group blocks, use the iterator:
/// ```ignore
/// for group in verbatim.group() {
///     println!("Subject: {}", group.subject.as_string());
///     for line in group.children.iter() {
///         // Process each line
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Verbatim {
    /// Subject line of the first group (backwards-compatible direct access)
    pub subject: TextContent,
    /// Content lines of the first group (backwards-compatible direct access)
    pub children: VerbatimContainer,
    /// Closing annotation shared by all groups
    pub closing_annotation: Annotation,
    /// Location spanning all groups and the closing annotation
    pub location: Range,
    /// Additional subject/content pairs beyond the first (for multi-group verbatims)
    additional_groups: Vec<VerbatimGroupItem>,
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
            additional_groups: Vec::new(),
        }
    }

    pub fn with_subject(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
            additional_groups: Vec::new(),
        }
    }

    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
            additional_groups: Vec::new(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }

    /// Attach additional verbatim group entries beyond the first pair.
    pub fn with_additional_groups(mut self, groups: Vec<VerbatimGroupItem>) -> Self {
        self.additional_groups = groups;
        self
    }

    /// Returns an iterator over each subject/content pair in the group order.
    pub fn group(&self) -> VerbatimGroupIter<'_> {
        VerbatimGroupIter {
            first_yielded: false,
            verbatim: self,
            rest: self.additional_groups.iter(),
        }
    }

    /// Returns the number of subject/content pairs held by this verbatim block.
    pub fn group_len(&self) -> usize {
        1 + self.additional_groups.len()
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
        // Visit all groups, not just the first
        for group in self.group() {
            super::super::traits::visit_children(visitor, group.children);
        }
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
            "VerbatimBlock('{}', {} groups, closing: {})",
            self.subject.as_string(),
            self.group_len(),
            self.closing_annotation.label.value
        )
    }
}

/// Stored representation of additional verbatim group entries
#[derive(Debug, Clone, PartialEq)]
pub struct VerbatimGroupItem {
    pub subject: TextContent,
    pub children: VerbatimContainer,
}

impl VerbatimGroupItem {
    pub fn new(subject: TextContent, children: Vec<ContentItem>) -> Self {
        Self {
            subject,
            children: VerbatimContainer::new(children),
        }
    }
}

/// Immutable view over a verbatim group entry.
#[derive(Debug, Clone)]
pub struct VerbatimGroupItemRef<'a> {
    pub subject: &'a TextContent,
    pub children: &'a VerbatimContainer,
}

/// Iterator over all subject/content pairs inside a verbatim block.
pub struct VerbatimGroupIter<'a> {
    first_yielded: bool,
    verbatim: &'a Verbatim,
    rest: slice::Iter<'a, VerbatimGroupItem>,
}

impl<'a> Iterator for VerbatimGroupIter<'a> {
    type Item = VerbatimGroupItemRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_yielded {
            self.first_yielded = true;
            return Some(VerbatimGroupItemRef {
                subject: &self.verbatim.subject,
                children: &self.verbatim.children,
            });
        }

        self.rest.next().map(|item| VerbatimGroupItemRef {
            subject: &item.subject,
            children: &item.children,
        })
    }
}
