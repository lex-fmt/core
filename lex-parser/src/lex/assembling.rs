//! Assembling module
//!
//!     The assembling stage processes parsed AST nodes to attach metadata and perform
//!     post-parsing transformations. Unlike the parsing stage which converts tokens to AST,
//!     assembling stages operate on the AST itself.
//!
//!     We do have a document ast node, but it's not yet complete. Annotations, which are
//!     metadata, are always attached to AST nodes, so they can be very targeted. Only with
//!     the full document in place we can attach annotations to their correct target nodes.
//!
//!     This is harder than it seems. Keeping Lex ethos of not enforcing structure, this needs
//!     to deal with several ambiguous cases, including some complex logic for calculating
//!     "human understanding" distance between elements.
//!
//! Current stages:
//!
//!     - `attach_root`: Wraps the built session tree in a [`Document`].
//!     - `attach_annotations`: Attaches annotations from content to AST nodes as metadata.
//!       See [attach_annotations](stages::attach_annotations) for details.

pub mod stages;

pub use stages::{AttachAnnotations, AttachRoot};
