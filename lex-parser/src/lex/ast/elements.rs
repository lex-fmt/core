//! Element-specific AST node definitions for the lex format
//!
//! The lex document structure is incredibly simple:
//! - Nested elements (all but paragraphs)
//! - All elements share a common structure:
//!
//! <lead>
//! <container> -> all children go here
//!     <children> ...
//! </container>
//! </lead>
//!

pub mod annotation;
pub mod blank_line_group;
pub mod container;
pub mod content_item;
pub mod definition;
pub mod document;
pub mod label;
pub mod list;
pub mod paragraph;
pub mod parameter;
pub mod session;
pub mod typed_content;
pub mod verbatim;
pub mod verbatim_line;

// Re-export all element types
pub use annotation::Annotation;
pub use blank_line_group::BlankLineGroup;
#[allow(deprecated)]
pub use container::Container as ContainerNode;
pub use content_item::ContentItem;
pub use definition::Definition;
pub use document::Document;
pub use label::Label;
pub use list::{List, ListItem};
pub use paragraph::{Paragraph, TextLine};
pub use parameter::Parameter;
pub use session::Session;
pub use typed_content::{ContentElement, ListContent, SessionContent, VerbatimContent};
pub use verbatim::Verbatim;
pub use verbatim_line::VerbatimLine;
