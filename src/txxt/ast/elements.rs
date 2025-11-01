//! Element-specific AST node definitions for the txxt format
//!
//! The txxt document structure is incredibly simple:
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
pub mod content_item;
pub mod definition;
pub mod document;
pub mod foreign;
pub mod label;
pub mod list;
pub mod paragraph;
pub mod parameter;
pub mod session;

// Re-export all element types
pub use annotation::Annotation;
pub use content_item::ContentItem;
pub use definition::Definition;
pub use document::Document;
pub use foreign::ForeignBlock;
pub use label::Label;
pub use list::{List, ListItem};
pub use paragraph::{Paragraph, TextLine};
pub use parameter::Parameter;
pub use session::Session;
