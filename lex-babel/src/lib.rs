//! Multi-format interoperability for Lex documents
//!
//!     This crate provides a uniform interface for converting between Lex AST and various document
//!     formats (Markdown, HTML, Pandoc JSON, etc.).
//!
//! Architecture
//!
//!     - Format trait: Uniform interface for all formats (parsing and/or serialization)
//!     - FormatRegistry: Centralized discovery and selection of formats
//!     - Format implementations: Concrete implementations for each supported format
//!
//!     This is a pure lib, that is , it powers the lex-cli but is shell agnostic, that is no code
//!     should be written that supposes a shell environment, be it to std print, env vars etc.
//!
//!     Format specific capabilities are implemented with the Format trait. formats should have a
//!     parse() and serialize() method, a name and file extensions. See the trait def [./format.rs ]
//!
//!     The file structure :
//!     .
//!     ├── error.rs
//!     ├── format.rs               # Format trait definition
//!     ├── registry.rs             # FormatTregistry for discovery and selection
//!     ├── formats
//!     │   ├── <format>
//!     │   │   ├── parser.rs       # Parser implementation
//!     │   │   ├── serializer.rs   # Serializer implementation
//!     │   │   └── mod.rs
//!     |   ├─  interop             # Shared conversion utilities
//!     ├── lib.rs
//!
//! Testing   
//!     tests
//!     └── <format>
//!         ├── <testname>.rs
//!         └── fixtures
//!         ├── <docname>.<format>
//!         ├── kitchensink.html
//!         ├── kitchensink.lex
//!         └── kitchensink.md
//!
//! note that rust does not by default discover tests in subdirectories, so we need to include these
//! in the mod.
//!
//! The Lex Format
//!
//!     The Lex format itself is implemented as a format, see ./formats/lex/mod.rs, which allows for
//!     a homogeneous API where all formats have identical interfaces:
//!
//!     Note that Lex is a more expressive format than most, which means that converting from is
//!     simple , but always lossy. In particular converting from requires some cosnideartion on how
//!     to best represent the author's intent.
//!
//!     This means that full format interop round tripping is not possible.
//!
//!
//! Implementation Principles
//!
//!     This, not being lex's core means that we will offload as much as possible to better, scpecialized creates for each format. the escope here is mainly to adapt the ast's from lex to the format or vice versa. For example we never write the serializer for , say markdown, but pass the AST to the mardown library.
//!     To support a format inbound, we write the format ast -> lex ast adapter. likewise, for outbound formats we wiill do the reverse, converting from the lex ast to the format's.
//!
//!     As much as possible, we will use rust crates, and avoid shelling out and having outside dependencies, but this can be innevitable as for pandoc.
//!
//! Format Selection
//!
//!     The choice for the formats is pretty sensible:
//!
//!     - HTML Output: should be self arguing, as it's the most common format for publishing and viewing.
//!     - Markdown: both in and to, as Mardown is the universal format for plain text editing.
//!     - XML: serializing Lex's is trivial and can be useful as a structured format for storage.
//!
//!     These are table stakes, that is a format that can't export to HTML, convert to markdown or lack a good semantic pure xml output is a non starter.
//!
//!
//!     For everything else, there is good arguments for a variety of formats. The one that has the strongest fit and use case is Latex, as Lex can be very useful for scientifc writing. But latex is complicated, and having pandoc in the pipeline allows us to serve reasonably well pretty much any other format.
//!
//!     This entails to only tree implementations. The hardest part of the work is about the mapping of nested to flat structures, and this code can be reused for all formats.
//!
pub mod error;
pub mod format;
pub mod formats;
pub mod registry;

pub mod ir;
pub mod mappings;

pub use error::FormatError;
pub use format::Format;
pub use registry::FormatRegistry;

/// Converts a lex document to the Intermediate Representation (IR).
///
/// # Information Loss
///
/// The IR is a simplified, semantic representation. The following
/// Lex information is lost during conversion:
/// - Blank line grouping (BlankLineGroup nodes)
/// - Source positions and token information
/// - Comment annotations at document level
///
/// For lossless Lex representation, use the AST directly.
pub fn to_ir(doc: &lex_parser::lex::ast::elements::Document) -> ir::nodes::Document {
    ir::from_lex::from_lex_document(doc)
}
