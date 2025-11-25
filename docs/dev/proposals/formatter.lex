:: title :: Proposal: Lex Formatter

1. Introduction

    Lex is a structured document format with rich semantics. This proposal introduces a Formatter: a serializer that converts the Lex AST back to properly formatted source text, enabling code formatting, round-trip transformations, and consistent document structure.

    The formatter completes the missing serialization path in the Lex format implementation, filling the gap identified in `lex-babel/src/formats/lex/mod.rs:45` where `serialize()` returns `NotSupported`.

2. Problem Statement

    Currently, Lex can parse documents into a rich AST, but cannot serialize that AST back to Lex source. This limits several important use cases:

    - No `lex format` command to normalize document structure
    - No round-trip testing (parse → modify → serialize)
    - No automatic formatting in editors via LSP
    - Convert command cannot output to Lex format
    - Programmatic document generation requires manual string construction

    Additionally, users need consistent formatting for:

    - Normalizing indentation (always 4 spaces, correct depth)
    - Controlling blank lines (sessions need 1 before/after title)
    - Standardizing list markers (normalize to `-` for bullets)
    - Ordering numbered lists (sequential 1. 2. 3.)
    - Removing trailing whitespace
    - Ensuring files end with newline

3. Proposed Design

    Build an AST-based formatter using the Visitor pattern that traverses the Document and generates properly formatted source text. This approach provides semantic awareness of element types and their formatting requirements.

    3.1. Why AST-Based Instead of Token-Based

        The existing detokenizer (`lex-parser/src/lex/token/formatting.rs`) works well for round-tripping token streams, but has limitations:

        - Operates only on lexer output, not AST
        - Cannot apply semantic rules (sessions need blank lines, lists have specific markers)
        - Cannot restructure based on element context
        - Cannot normalize blank line groups or reorder list markers

        An AST-based approach enables semantic formatting:

        - Knows element types and their requirements
        - Can apply context-specific rules (blank lines before sessions)
        - Can normalize structure (collapse multiple blanks, order lists)
        - Leverages existing Visitor infrastructure

    3.2. Why Not IR-Based

        The Intermediate Representation loses critical information:

        - Blank line grouping (how many blanks, where)
        - Source positions and ranges
        - Document-level annotations
        - Exact indentation structure

        The formatter needs this information to produce correct, idiomatic Lex output.

4. Architecture Overview

    The formatter integrates into the existing transform pipeline as a serialization stage:

        Source → Parse → AST → Serialize → Formatted Source
    :: diagram ::

    Key components:

    - `LexSerializer`: Visitor implementation that generates formatted text
    - `FormattingRules`: Configuration for formatting behavior
    - `AST_TO_LEX_STRING`: Transform in the standard pipeline
    - CLI integration: `lex format` command (outputs to stdout)
    - LSP integration: `textDocument/formatting` capability
    - Babel integration: Completes `LexFormat::serialize()`

5. Core Components

    5.1. The Serializer Visitor

        Location: `lex-babel/src/formats/lex/serializer.rs`

        Implements the Visitor trait to traverse the AST and build formatted output. Maintains state for indentation level, previous element type, and blank line tracking.

        Responsibilities:

        - Traverse AST using visitor pattern
        - Track indentation depth (increment for children)
        - Manage blank line insertion (based on element rules)
        - Format element-specific syntax (session titles, list markers, etc.)
        - Apply formatting rules from configuration
        - Accumulate output string

        Key methods:

        - `visit_session()`: Write title, manage blank lines, indent children
        - `visit_paragraph()`: Write lines with current indentation
        - `visit_list()`: Determine marker style, format items
        - `visit_verbatim()`: Preserve literal content with markers
        - `visit_annotation()`: Format data line and parameters
        - `write_line()`: Append indentation + text + newline
        - `ensure_blank_lines()`: Manage blank line counts

    5.2. Formatting Rules Configuration

        Location: `lex-babel/src/formats/lex/formatting_rules.rs`

        Encapsulates all formatting decisions as configurable parameters. Allows different formatting styles without changing serializer logic.

        Configuration options:

        - `session_blank_lines_before`: Count before session title (default: 1)
        - `session_blank_lines_after`: Count after session title (default: 1)
        - `normalize_seq_markers`: Standardize markers (default: true)
        - `unordered_seq_marker`: Default bullet char (default: '-')
        - `max_blank_lines`: Collapse excess blanks (default: 2)
        - `indent_string`: Indentation unit (default: "    ")
        - `preserve_trailing_blanks`: Keep blanks at document end (default: false)
        - `normalize_verbatim_markers`: Use consistent `::` markers (default: true)

        Default implementation provides opinionated, consistent formatting. Future work can add configuration file support.

    5.3. Transform Integration

        Location: `lex-parser/src/lex/transforms/stages/serialize.rs`

        Create a `SerializeToLex` stage implementing `Runnable<Document, String>`:

        - Takes a Document AST
        - Creates LexSerializer with default rules
        - Visits the document
        - Returns formatted string

        Add to standard transforms:

        - `AST_TO_LEX_STRING`: Static transform for formatting
        - Composable with other transforms via `.then()`

        This follows the established pattern from `LEXING`, `STRING_TO_AST`, etc.

6. Formatting Rules

    The formatter applies these normalization rules:

    6.1. Indentation

        - Always use 4 spaces (never tabs)
        - Correct depth based on nesting level
        - Each child increments depth by 1
        - Root session children at depth 1

    6.2. Blank Lines

        - Sessions: 1 blank before title, 1 after (unless document start/end)
        - Collapse multiple consecutive blanks to maximum configured
        - No trailing blanks at document end (unless configured)
        - Preserve blank line groups within constraints

    6.3. List Markers

        - Normalize bullets to `-` (unless configured otherwise)
        - Ordered lists use sequential numbering: `1.` `2.` `3.`
        - Alphabetical: `a.` `b.` `c.` (preserve if present)
        - Roman numerals: `i.` `ii.` `iii.` (preserve if present)
        - Consistent spacing after markers

    6.4. Whitespace

        - No trailing whitespace on lines
        - File ends with single newline
        - No leading whitespace before root content

    6.5. Special Elements

        - Verbatim: Preserve content exactly, normalize markers to `::`
        - Annotations: Format as `:: label param=value ::`
        - Definitions: `Term:` with proper indentation
        - Respect inline formatting (preserve as-is)

7. Implementation Strategy

    Build incrementally with comprehensive testing at each phase:

    7.1. Phase 1: Basic Serializer

        Goal: AST → source round-trip with no formatting rules

        Tasks:

        - Create `LexSerializer` struct with Visitor implementation
        - Implement visit methods for all element types
        - Handle indentation tracking
        - Handle text content and inline preservation
        - Write basic unit tests for each element type

        Verification: Parse → Serialize → Parse produces equivalent AST

    7.2. Phase 2: Formatting Rules

        Goal: Apply normalization rules

        Tasks:

        - Create `FormattingRules` configuration
        - Implement blank line normalization
        - Implement list marker normalization
        - Implement indentation normalization
        - Implement whitespace cleanup

        Verification: Formatted output matches expected canonical form

    7.3. Phase 3: Transform Integration

        Goal: Wire into transform pipeline

        Tasks:

        - Create `SerializeToLex` transform stage
        - Add `AST_TO_LEX_STRING` to standard transforms
        - Update `LexFormat::serialize()` to use serializer

        Verification: FormatRegistry can serialize to lex format

    7.4. Phase 4: CLI Integration

        Goal: Enable `lex format` command

        Tasks:

        - Add format subcommand to CLI
        - Wire to transform pipeline
        - Output to stdout only
        - Support `--extra-` parameters for rules

        Verification: CLI can format files and output to stdout

    7.5. Phase 5: LSP Integration

        Goal: Enable editor formatting

        Tasks:

        - Add `format_document` to FeatureProvider
        - Implement `textDocument/formatting` handler
        - Return TextEdit for entire document
        - Use default formatting rules

        Verification: Editor can format on save or on command

8. Testing Strategy

    Testing follows strict Lex testing rules:

    8.1. Unit Tests (Core Formatting Logic)

        Location: `lex-babel/src/formats/lex/serializer.rs` (tests module)

        What to test:

        - Each element type serialization (paragraph, session, list, etc.)
        - Each formatting rule (blank lines, indentation, markers)
        - Edge cases (empty elements, deeply nested, max blanks)
        - Inline preservation (formatting, references, math, code)
        - All variations from spec files

        Test pattern:

        - Use Lexplore to load verified AST nodes
        - Serialize node to string
        - Assert specific formatting properties
        - Re-parse and assert AST equivalence

        Example:

            #[test]
            fn test_session_blank_lines() {
                let session = Lexplore::get_session(1);
                let formatted = serialize_element(&session);

                assert!(formatted.starts_with("\n")); // blank before
                let lines: Vec<_> = formatted.lines().collect();
                assert_eq!(lines[1], ""); // blank after title
            }
        :: rust ::

    8.2. Formatting Rule Tests

        Location: `lex-babel/src/formats/lex/formatting_rules.rs` (tests module)

        Test each rule in isolation:

        - `test_normalize_seq_markers_bullets()`
        - `test_normalize_seq_markers_numbered()`
        - `test_collapse_blank_lines()`
        - `test_indentation_depth()`
        - `test_trailing_whitespace_removal()`

    8.3. Integration Tests (Wiring)

        Location: `lex-babel/src/formats/lex/mod.rs` (tests module)

        What to test:

        - `LexFormat::serialize()` calls serializer correctly
        - Transform pipeline includes serialize stage
        - CLI commands wire to correct functions
        - Parameters pass through correctly

        NOT what to test:

        - Specific formatting output (unit tests cover this)
        - AST structure (parser tests cover this)

        Example:

            #[test]
            fn test_format_serialize_integration() {
                let format = LexFormat;
                let doc = Lexplore::get_document(1);

                // Just verify it works and returns string
                let result = format.serialize(&doc);
                assert!(result.is_ok());
                assert!(!result.unwrap().is_empty());
            }
        :: rust ::

    8.4. Round-Trip Tests

        Location: `lex-babel/src/formats/lex/serializer.rs` (tests module)

        Parse → Serialize → Parse should produce equivalent AST:

            #[test]
            fn test_round_trip_all_elements() {
                for element_type in [Paragraph, Session, List, ...] {
                    for doc_num in element_type.available_samples() {
                        let doc1 = Lexplore::get(element_type, doc_num);
                        let formatted = serialize(&doc1);
                        let doc2 = parse(&formatted);
                        assert_ast_equivalent(&doc1, &doc2);
                    }
                }
            }
        :: rust ::

    8.5. Line-Based Diff Test Utility

        Location: `lex-parser/src/lex/testing/text_diff.rs` (new file)

        Problem: When comparing source strings, `assert_eq!` produces unhelpful output. A single blank line difference causes every subsequent line to be marked as different, obscuring the actual issue.

        Solution: Create a line-based diff utility that:

        - Compares strings line by line
        - Reports exact line numbers where differences occur
        - Shows what changed (added, removed, modified)
        - Provides clear, actionable error messages
        - Handles blank line differences intelligently

        Implementation:

            pub fn assert_text_eq(actual: &str, expected: &str, context: &str) {
                let actual_lines: Vec<_> = actual.lines().collect();
                let expected_lines: Vec<_> = expected.lines().collect();

                let diff = diff_lines(&actual_lines, &expected_lines);

                if !diff.is_empty() {
                    let mut msg = format!("Text comparison failed: {}\n\n", context);
                    for change in diff {
                        match change {
                            LineDiff::Added(line_num, line) =>
                                msg.push_str(&format!("+ Line {}: {}\n", line_num, line)),
                            LineDiff::Removed(line_num, line) =>
                                msg.push_str(&format!("- Line {}: {}\n", line_num, line)),
                            LineDiff::Modified(line_num, old, new) =>
                                msg.push_str(&format!("~ Line {}: '{}' → '{}'\n", line_num, old, new)),
                        }
                    }
                    panic!("{}", msg);
                }
            }
        :: rust ::

        Usage in tests:

            #[test]
            fn test_session_formatting() {
                let session = Lexplore::get_session(1);
                let formatted = serialize(&session);
                let expected = "Introduction:\n\n    Content\n";

                assert_text_eq(&formatted, expected, "session-01-flat-simple");
                // On failure, shows:
                // Text comparison failed: session-01-flat-simple
                //
                // + Line 3: (blank line added)
                // ~ Line 4: '  Content' → '    Content' (indentation fixed)
            }
        :: rust ::

        Benefits:

        - Pinpoints exact differences (line numbers, content)
        - Distinguishes blank line issues from content issues
        - Shows indentation problems clearly
        - Provides context for failures
        - Much faster debugging than `assert_eq!`

        This utility should be exported from `lex-parser/src/lex/testing/mod.rs` for use in all formatting tests.

9. Test Coverage Requirements

    The formatter must handle all variations from specs:

    9.1. Paragraphs

        - Single line (`paragraph-01-flat-oneline`)
        - Multiple lines (`paragraph-02-flat-multiline`)
        - Special characters (`paragraph-03-flat-special-chars`)
        - Numbers (`paragraph-04-flat-numbers`)
        - With inlines (formatting, references)

    9.2. Sessions

        - Simple title (`session-01-flat-simple`)
        - Numbered title (`session-02-flat-numbered-title`)
        - Alphanumeric (`session-04-flat-alphanumeric-title`)
        - Nested sessions (`session-05-nested-simple`)
        - Multiple paragraphs (`session-03-flat-multiple-paragraphs`)
        - Blank line edge cases (`session-13-blankline-issue`)

    9.3. Lists

        - Dash markers (`list-01-flat-simple-dash`)
        - Numbered (`list-02-flat-numbered`)
        - Alphabetical (`list-03-flat-alphabetical`)
        - Mixed markers (`list-04-flat-mixed-markers`)
        - Parenthetical (`list-05-flat-parenthetical`)
        - Roman numerals (`list-06-flat-roman-numerals`)
        - Nested (`list-07-nested-simple`)
        - Deep nesting (`list-10-nested-deep-only`)
        - With paragraphs (`list-08-nested-with-paragraph`)

    9.4. Definitions

        - Simple (`definition-01-flat-simple`)
        - Multiple paragraphs (`definition-02-flat-multi-paragraph`)
        - With lists (`definition-03-flat-with-list`)
        - Nested definitions (`definition-06-nested-definitions`)

    9.5. Verbatim

        - Simple code (`verbatim-01-flat-simple-code`)
        - With caption (`verbatim-02-flat-with-caption`)
        - With parameters (`verbatim-03-flat-with-params`)
        - Marker form (`verbatim-04-flat-marker-form`)
        - Special characters (`verbatim-05-flat-special-chars`)
        - Empty blocks (`verbatim-10-flat-simple-empty`)
        - Multiple groups (`verbatim-11-group-shell`)

    9.6. Annotations

        - Flat marker simple (`annotation-01-flat-marker-simple`)
        - With parameters (`annotation-02-flat-marker-with-params`)
        - Inline text (`annotation-03-flat-inline-text`)
        - Block content (`annotation-05-flat-block-paragraph`)
        - Nested complex (`annotation-10-nested-complex`)
        - Quoted parameters (`annotation-11-quoted-parameter`)
        - Document-level (`annotation-27-attachment-example-l-multiple-document-level`)

    9.7. Documents

        - Simple document (`XXX-document-simple`)
        - Complex document (`XXX-document-tricky`)
        - Benchmark documents (all)
        - Trifecta documents (all)

    9.8. Edge Cases

        - Empty document
        - Only blank lines
        - Maximum nesting depth
        - Multiple consecutive blank lines
        - Mixed element types
        - All inline types preserved
        - Unicode and special punctuation

10. Integration Points

    10.1. CLI Integration

        Location: `lex-cli/src/main.rs`

        The formatter integrates through the convert command, not a separate format command. When converting lex → lex, serialization happens automatically.

        Usage:

            lex file.lex --to lex                 # Format and output to stdout
            lex file.lex --to lex -o clean.lex    # Format to file
            lex file.md --to lex                  # Convert markdown to formatted lex
        :: shell ::

        Implementation uses FormatRegistry:

            fn handle_convert_command(input: &str, from: &str, to: &str, ...) {
                let registry = FormatRegistry::default();
                let source = fs::read_to_string(input)?;
                let doc = registry.parse(&source, from)?;
                let output = registry.serialize(&doc, to)?;  // Uses LexFormat::serialize
                print!("{}", output);
            }
        :: rust ::

    10.2. LSP Integration

        Location: `lex-lsp/src/features/formatting.rs` (new file)

        Add formatting capability to the language server:

        - Implement `FeatureProvider::format_document()`
        - Handle `textDocument/formatting` request
        - Return `TextEdit` replacing entire document
        - Use default `FormattingRules`

        Editors can then:

        - Format on save
        - Format on command
        - Format selection (future work)

    10.3. Babel Integration

        Location: `lex-babel/src/formats/lex/mod.rs`

        Complete the `Format` trait implementation:

            impl Format for LexFormat {
                fn supports_serialization(&self) -> bool {
                    true  // Changed from false
                }

                fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
                    let rules = FormattingRules::default();
                    let serializer = LexSerializer::new(rules);
                    serializer.serialize(doc)
                }
            }
        :: rust ::

        This enables lex as a conversion target in FormatRegistry.

    10.4. Transform Pipeline Integration

        Location: `lex-parser/src/lex/transforms/standard.rs`

        Add serialization transform to standard pipeline:

            pub static AST_TO_LEX_STRING: Lazy<Transform<Document, String>> =
                Lazy::new(|| {
                    Transform::from_fn(|doc| {
                        let serializer = LexSerializer::new(FormattingRules::default());
                        serializer.serialize(&doc)
                    })
                });
        :: rust ::

        This allows direct use in transforms:

            let formatted = AST_TO_LEX_STRING.run(document)?;
        :: rust ::

11. File Structure

    The formatter implementation spans multiple crates:

    11.1. Core Serializer (lex-babel)

        lex-babel/src/formats/lex/
        ├── mod.rs                  # LexFormat implementation (update serialize())
        ├── serializer.rs           # LexSerializer visitor (NEW)
        └── formatting_rules.rs     # FormattingRules config (NEW)
    :: tree ::

    11.2. Transform Stage (lex-parser)

        lex-parser/src/lex/transforms/
        ├── stages/
        │   └── serialize.rs        # SerializeToLex stage (NEW)
        └── standard.rs             # Add AST_TO_LEX_STRING (UPDATE)
    :: tree ::

        lex-parser/src/lex/testing/
        ├── mod.rs                  # Export assert_text_eq (UPDATE)
        └── text_diff.rs            # Line-based diff utility (NEW)
    :: tree ::

    11.3. CLI (lex-cli)

        No new files needed. The convert command already wires through FormatRegistry, which will use the updated `LexFormat::serialize()`.

    11.4. LSP (lex-lsp)

        lex-lsp/src/features/
        └── formatting.rs           # Document formatting feature (NEW)
    :: tree ::

        Update `lex-lsp/src/lib.rs` to register formatting capability.

12. Success Criteria

    The formatter is complete when:

    - All element types serialize correctly with proper syntax
    - All formatting rules apply consistently
    - Round-trip tests pass for all spec files
    - `lex file.lex --to lex` outputs formatted source
    - LSP `textDocument/formatting` works in editors
    - `FormatRegistry.serialize(doc, "lex")` succeeds
    - Unit test coverage ≥ 95% for serializer
    - Integration tests verify wiring, not output
    - Documentation explains formatting rules and configuration

14. Design Decisions

    These questions were resolved during design:

    14.1. Malformed AST Handling

        Question: How to handle malformed AST (missing ranges, invalid structure)?

        Decision: Malformed AST should not exist. The formatter assumes it receives valid AST from the parser. If the AST is invalid, that's a parser bug, not a formatter concern.

        Implication: Formatter can assert on required AST properties without defensive null checks.

    14.2. Parse Error Correction

        Question: Should formatter fix parse errors or preserve as-is?

        Decision: The formatter should ideally fix formatting issues, but cannot fix parse errors. It operates on the AST, not the source text. If the source had parse errors, those would prevent AST generation in the first place.

        Implication: Formatter only fixes formatting (indentation, blank lines, markers), not structural errors.

    14.3. Annotation Label Handling

        Question: How to handle custom/unknown annotation types?

        Decision: Annotation labels are user-defined and only need to be well-formed. The formatter serializes any annotation correctly based on its AST structure (label, parameters, content), regardless of the label value.

        Implication: No whitelist or validation of annotation labels. Format all annotations uniformly using their AST data.

    14.4. List Marker Normalization

        Question: Should list marker normalization preserve original style or always normalize?

        Decision: Use the list decoration property from the List AST node. Each list has a decoration property that specifies the marker style. For nested lists, each level can have its own style.

        The formatter must:
        - Respect the decoration property from AST
        - Normalize ordering (ensure sequential: 1. 2. 3., not 1. 3. 5.)
        - Apply same rules to decorated numbered sessions

        Implication: Read `list.decoration` field, generate appropriate markers, ensure correct sequence.

    14.5. Long Line Handling

        Question: How to handle very long lines (word wrap, preserve, or error)?

        Decision: Preserve long lines as-is. No automatic word wrapping or line breaking.

        Implication: Output lines can be arbitrarily long. Users manage line length manually.

    14.6. Blank Line Rules

        Question: Should blank line rules differ for document-level vs nested sessions?

        Decision: No. Blank line rules are consistent regardless of nesting depth. All sessions get the same treatment (1 blank before title, 1 after).

        Implication: Single set of blank line rules applies universally.
