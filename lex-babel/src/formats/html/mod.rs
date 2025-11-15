//! HTML format implementation
//!
//! Strategy: Direct Lex AST → HTML generation (one-way only)
//!
//! # Data Model
//!
//! Semantic HTML generation mappings:
//!
//! | Lex Element | HTML Element | Notes |
//! |-------------|--------------|-------|
//! | Session | `<section>` with `<h1>`-`<h6>` | Heading level based on nesting depth |
//! | Paragraph | `<p>` | Direct mapping |
//! | List | `<ul>` or `<ol>` | Ordered/unordered based on list type |
//! | ListItem | `<li>` | List item |
//! | Definition | `<dl><dt><dd>` | HTML definition list |
//! | VerbatimBlock | `<pre><code class="language-X">` | Code block with language class |
//! | VerbatimLine | `<code>` | Inline code |
//! | Annotation | `<aside>` or custom data attributes | Configurable strategy |
//!
//! # Architecture
//!
//! The HTML formatter generates semantic HTML directly from the Lex AST through
//! traversal. This is a serialization-only format (no parsing from HTML to Lex).
//!
//! ## Design Decisions
//!
//! - Semantic HTML: Uses meaningful HTML5 elements (`<section>`, `<article>`, etc.)
//!   rather than generic `<div>` containers
//! - Heading Levels: Automatically calculated based on Session nesting depth
//! - CSS Classes: Minimal by default, with option to add custom classes
//! - Accessibility: Generated HTML should follow ARIA best practices
//!
//! # Implementation Notes
//!
//! - Consider using a HTML builder utility from `interop::html` module
//! - Support configurable options via `HtmlOptions` struct:
//!   - Include CSS classes
//!   - Custom class prefixes
//!   - Annotation rendering strategy
//!   - Pretty-print vs minified output
//!
//! # Testing
//!
//! - Unit tests for each AST element → HTML mapping
//! - Integration tests for complete documents
//! - Snapshot tests for complex document structures
//!
