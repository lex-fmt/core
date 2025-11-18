# Cargo Documentation Recommendations for lex-parser

This document provides recommendations for where design documentation (why and how) should be placed based on `docs/dev/guides/on-all-of-lex.lex`.

## Overview

The goal is to add **design documentation** (not API docs) that explains the "why" and "how" behind the code, using the verified content from the guide as reference. The documentation should go at the module level (using `//!`) to provide context about purpose, structure, and usage.

---

## Section Mappings

### 1. Structure (Guide Sections 1.1-1.3)

#### `lex-parser/src/lex/ast/elements.rs`

**Sections to document: 1.3 Elements, 6. Structure, Children, Indentation and the AST**

This is the entry point for all AST element types. Should cover:

- The four types of elements (blocks, containers, inlines, components)
- How indentation manifests as container nodes
- The pattern: `<head> <blank-line>? <indent> <content> <dedent> <tail>?`
- Why titles/subjects are siblings, not parents, of their content
- The parsing tables showing head/tail/blank-line/indent patterns for each element type

**Key points:**

- Indentation = container node = where elements hold children
- Sessions: title is a child, content is in session.content container
- Lists: items don't indent, nested lists are in item.content container
- Special handling for sessions requiring preceding blank lines

#### `lex-parser/src/lex/ast.rs`

**Sections to document: 1.1 Document and Sessions, 1.2 Nesting**

Should cover:

- Document structure and root session
- Nesting rules and restrictions
- Type-safe containers architecture

#### `lex-parser/src/lex/ast/elements/document.rs`

**Sections to document: 1.1 Document and Sessions**

Specific to document structure and session hierarchy.

#### `lex-parser/src/lex/ast/elements/session.rs`

**Sections to document: 1.1 Document and Sessions, 6 (special handling)**

Document the session-specific parsing rules, especially the preceding blank line requirement and synthetic token injection.

---

### 2. Grammar (Guide Section 2)

#### `lex-parser/src/lex/token.rs`

**Sections to document: 2.3 Tokens**

Overview of token layers:

- Structural tokens: Indent, Dedent
- Core tokens: character/word level from logos
- Line tokens: groups of core tokens
- Line container tokens: tree representation
- Synthetic tokens: context from parent to children

#### `lex-parser/src/lex/token/core.rs`

**Sections to document: 2.3 Tokens (core tokens)**

Core token definitions from logos lexer.

#### `lex-parser/src/lex/token/line.rs`

**Sections to document: 2.3 Tokens (line tokens), 3.2 Lines**

Line token types and classification. The LineType enum is the definitive set.

#### `lex-parser/src/lex/token/to_line_container.rs`

**Sections to document: 2.3 Tokens (line containers), 4 Parser Design**

How line tokens are grouped into a hierarchical tree structure for parsing.

#### `lex-parser/src/lex/lexing/line_classification.rs`

**Sections to document: 2.3 Tokens, 3.2 Lines**

Line classification logic and the order of line categorization.

#### `lex-parser/src/lex/ast/elements/blank_line_group.rs`

**Sections to document: 2.2 Blank Lines**

Blank line semantics and grouping.

---

### 3. Syntax (Guide Section 3)

#### `lex-parser/src/lex/ast/elements/data.rs`

**Sections to document: 3.1.1 Data Nodes**

Data node syntax: `:: label params?`

#### `lex-parser/src/lex/ast/elements/label.rs` and `parameter.rs`

**Sections to document: 3.1.1 Data Nodes**

Component elements used in metadata.

#### `lex-parser/src/lex/parsing/parser/grammar.rs`

**Sections to document: 3.1 Markers, 3.2 Lines**

Grammar patterns and marker matching logic.

---

### 4. Parser Design (Guide Section 4)

#### `lex-parser/src/lex.rs`

**Sections to document: 4 Parser Design**

Top-level module overview explaining the multi-stage design:

1. Semantic Indentation
2. Line grouping
3. Tree building (LineContainer)
4. Context injection
5. Parsing by level

Why this design instead of standard libraries (stateful, recursive, line-based, indentation-significant).

#### `lex-parser/src/lib.rs`

**Sections to document: 4 Parser Design (high-level)**

Entry point overview of the parsing architecture.

---

### 5. Parsing End To End (Guide Section 5)

#### `lex-parser/src/lex/lexing.rs`

**Sections to document: 5.1 Lexing (overview), 5.1.1 Base Tokenization**

Complete lexing pipeline overview:

- TokenStream transformations
- Source token preservation
- Byte range preservation for location tracking

#### `lex-parser/src/lex/lexing/base_tokenization.rs`

**Sections to document: 5.1.1 Base Tokenization**

Logos lexer usage - declarative, no custom logic.

#### `lex-parser/src/lex/lexing/transformations/semantic_indentation.rs`

**Sections to document: 5.1.2 Semantic Indentation**

State machine that tracks indentation levels and emits Indent/Dedent events. Indent stores source tokens; Dedent is synthetic.

#### `lex-parser/src/lex/lexing/line_grouping.rs`

**Sections to document: 5.1.3 Line Grouping**

Splits tokens by line breaks, classifies lines, handles structural tokens.

#### `lex-parser/src/lex/parsing.rs`

**Sections to document: 5 Parsing End To End (overview)**

Complete pipeline overview: lexing → analysis → building.

#### `lex-parser/src/lex/parsing/engine.rs`

**Sections to document: 5.2 Parsing (Semantic Analysis)**

Grammar pattern matching using regex. IR nodes carry node type + tokens. Separation of semantic analysis from AST building.

#### `lex-parser/src/lex/building.rs`

**Sections to document: 5.3 AST Building**

Three-layer architecture:

1. Token normalization
2. Data extraction (text, byte ranges)
3. AST creation (with location calculation)

#### `lex-parser/src/lex/building/location.rs`

**Sections to document: 5.3 AST Building (location tracking)**

Byte range → line:column conversion. Location aggregation from child nodes.

#### `lex-parser/src/lex/building/ast_tree.rs`

**Sections to document: 5.3 AST Building (AST tree construction)**

IR nodes → AST nodes conversion. Document node creation.

#### `lex-parser/src/lex/assembling.rs`

**Sections to document: 5.4 Document assembly**

Post-parsing transformations on the AST. Annotation attachment.

#### `lex-parser/src/lex/assembling/stages/attach_annotations.rs`

**Sections to document: 5.4 Document assembly, 6 (annotation handling)**

Attachment rules, distance calculation, ambiguous cases.

#### `lex-parser/src/lex/assembling/stages/attach_annotations/distance.rs`

**Sections to document: 5.4 Document assembly (distance calculation)**

"Human understanding" distance between elements for annotation attachment.

#### `lex-parser/src/lex/inlines.rs`

**Sections to document: 5.5 Inline Parsing (overview)**

Inline parsing overview - simpler than block parsing (formal start/end tokens, no structure).

#### `lex-parser/src/lex/inlines/parser.rs`

**Sections to document: 5.5 Inline Parsing**

Declarative engine processing element declarations. Flat transformations vs callbacks for complex inlines.

---

### 6. Structure, Children, Indentation and the AST (Guide Section 6)

#### Already covered above under `lex-parser/src/lex/ast/elements.rs`

This section is primarily about AST structure and should go in the elements module.

---

### 7. Verbatim Elements (Guide Section 7)

#### `lex-parser/src/lex/ast/elements/verbatim.rs`

**Sections to document: 7 Verbatim Elements (complete section)**

Should cover:

- Why verbatim is the most complex element
- Parsing must come first (lest content break structure)
- End marker identification (data node at same indentation)
- In-flow mode: indentation wall concept
- Full-width mode: column 2 wall, why not column 1
- Mode determination from first content line
- Verbatim groups (multiple subject/content pairs)

#### `lex-parser/src/lex/parsing/parser/builder/builders/verbatim.rs`

**Sections to document: 7.1 Parsing Verbatim Blocks**

The stateful parsing logic for verbatim blocks.

---

### 8. Testing (Guide Section 8)

#### `lex-parser/src/lex/testing.rs`

**Sections to document: 8 Testing (complete section)**

Already has excellent documentation. Could enhance with:

- Why we can't rely on reference parsers or established body of text
- Why ad-hoc test strings are problematic
- The spec sample files organization

#### `lex-parser/src/lex/testing/lexplore.rs`

**Sections to document: 8.1 The Spec Sample Files**

How Lexplore loads from official sample files in various formats.

#### `lex-parser/src/lex/testing/ast_assertions.rs`

**Sections to document: 8.2 The AST assertions**

Why manual AST walking tests are insufficient. How the fluent API helps with spec changes.

---

## Additional Module Documentation

### `lex-parser/src/lex/transforms.rs`

Should document the transform pipeline architecture (already has good docs).

### `lex-parser/src/lex/formats.rs`

Should document output format registry and detokenization.

### `lex-parser/src/lex/loader.rs`

Should document DocumentLoader convenience API.

---

## Priority Recommendations

**High Priority** (Core understanding):

1. `lex-parser/src/lex/ast/elements.rs` - Sections 1.3, 6
2. `lex-parser/src/lex.rs` - Section 4
3. `lex-parser/src/lex/parsing.rs` - Section 5 overview
4. `lex-parser/src/lex/lexing.rs` - Section 5.1 overview

**Medium Priority** (Important details):

1. `lex-parser/src/lex/ast/elements/verbatim.rs` - Section 7
2. `lex-parser/src/lex/parsing/engine.rs` - Section 5.2
3. `lex-parser/src/lex/building.rs` - Section 5.3
4. `lex-parser/src/lex/assembling.rs` - Section 5.4

**Lower Priority** (Implementation details):

1. Individual transformation modules
2. Builder modules
3. Helper utilities

---

## Documentation Style Notes

- Use `//!` for module-level documentation
- Focus on "why" and "how", not just "what"
- Reference guide sections when appropriate
- Include examples from the guide
- Explain design decisions and trade-offs
- Note where content touches multiple parts (cross-reference)
