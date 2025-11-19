//! Defines the flat event stream representation of a document.

use crate::ir::nodes::InlineContent;

/// Represents a single event in the document stream.
///
/// This enum is used to represent a document as a flat sequence of events,
/// which is useful for stream-based processing and conversion between formats.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    StartDocument,
    EndDocument,
    StartHeading(usize),
    EndHeading(usize),
    StartParagraph,
    EndParagraph,
    StartList {
        ordered: bool,
    },
    EndList,
    StartListItem,
    EndListItem,
    StartDefinition,
    EndDefinition,
    StartDefinitionTerm,
    EndDefinitionTerm,
    StartDefinitionDescription,
    EndDefinitionDescription,
    StartVerbatim(Option<String>),
    EndVerbatim,
    StartAnnotation {
        label: String,
        parameters: Vec<(String, String)>,
    },
    EndAnnotation {
        label: String,
    },
    Inline(InlineContent),
}
