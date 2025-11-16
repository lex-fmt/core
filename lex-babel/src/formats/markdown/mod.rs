//! Markdown format implementation
//!
//! Data Model:
//!
//!     Note that, Lex being more expressive that markdown, a few mappings are needed.
//!     | Markdown  | Lex | Notes |
//!     | Table ] Verbatim | with mardown.table as lex_babel
//!     | Header | Session \ the heading level is the session level |
//!     | Links |  URL Reference | The anchor is the word preceding the link \
//!
//!     There is a fundamental mismatch between markdown's flat model and lex's hierarchical.
//!     This means that the parsing code must assemble a session's content from the flat markdown
//!  tokens infering the 2d structure from the 1d nodes is not possible, so we will use heuristics
//!  to guess, but keeping a stack with session levels and infering a tree from that.
//!
//!
//!     Lists are the only markdown element that are truly nested, so they should be handled easily.
//!
//!
//! Library
//!
//!     We will use the comrak crate to handle parsing and serialization. IT's choice comes from
//!  being a single create for both purposes, that is feature-rich, robust and well maintained.
//!
//!
//! Testing
//!
//!     While we will have e2e string tests, the core logic for the mapping (which gets complicated)
//!  should bw unit tested on the AsTs alone.
//!     Note that the lex-parser crate has a robust testing toolset, including ast assertions that
//!  can verify a number of ast nodes and their data in a fluent way, use it. This also shields us
//!  from ast changes breaking every test.
