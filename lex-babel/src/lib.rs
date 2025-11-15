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

pub mod error;
pub mod format;
pub mod formats;
pub mod registry;

pub use error::FormatError;
pub use format::Format;
pub use registry::FormatRegistry;
