//! Verbatim block element
use super::super::range::{Position, Range};
use super::super::text_content::TextContent;
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::container::VerbatimContainer;
use super::content_item::ContentItem;
use super::typed_content::VerbatimContent;
use std::fmt;
use std::slice;
/// Represents the mode of a verbatim block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbatimBlockMode {
    /// The block's content is indented relative to the subject line.
    Inflow,
    /// The block's content starts at a fixed, absolute column.
    Fullwidth,
}
/// A verbatim block represents content from another format/system.
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
    /// The rendering mode of the verbatim block.
    pub mode: VerbatimBlockMode,
    /// Additional subject/content pairs beyond the first (for multi-group verbatims)
    additional_groups: Vec<VerbatimGroupItem>,
}
impl Verbatim {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }
    pub fn new(
        subject: TextContent,
        children: Vec<VerbatimContent>,
        closing_annotation: Annotation,
        mode: VerbatimBlockMode,
    ) -> Self {
        Self {
            subject,
            children: VerbatimContainer::from_typed(children),
            closing_annotation,
            location: Self::default_location(),
            mode,
            additional_groups: Vec::new(),
        }
    }
    pub fn with_subject(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
            mode: VerbatimBlockMode::Inflow,
            additional_groups: Vec::new(),
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject: TextContent::from_string(subject, None),
            children: VerbatimContainer::empty(),
            closing_annotation,
            location: Self::default_location(),
            mode: VerbatimBlockMode::Inflow,
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
        let group_count = self.group_len();
        let group_word = if group_count == 1 { "group" } else { "groups" };
        write!(
            f,
            "VerbatimBlock('{}', {} {}, closing: {})",
            self.subject.as_string(),
            group_count,
            group_word,
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
    pub fn new(subject: TextContent, children: Vec<VerbatimContent>) -> Self {
        Self {
            subject,
            children: VerbatimContainer::from_typed(children),
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
