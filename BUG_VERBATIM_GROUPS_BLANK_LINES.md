# Bug: Verbatim Groups Not Parsed Correctly with Blank Lines Between Pairs

**Status**: Open  
**Priority**: High  
**Related**: PR #207, Task 61-verbatim-groups-iteration

## Description

The verbatim group parser fails to correctly recognize multiple subject/content pairs when there are blank lines between pairs. Instead of parsing them as a single verbatim group, the parser treats subsequent subjects as separate sessions or definitions.

## Key Insight

**This is NOT about column position** - the ambiguity between verbatim blocks and sessions exists at ANY indentation level. The solution is parse priority: verbatim blocks are tried before sessions, so if something could be both, it should be parsed as a verbatim block.

**Before groups**: The parser looked for `subject → content → closing annotation`

**With groups**: The parser now looks for `(subject → content)+ → closing annotation` (multiple subjects at same indentation before the annotation)

Groups don't introduce new ambiguity - this was already a problem. The only change is recognizing multiple subject lines at the same indentation level before the closing annotation.

## Expected Behavior

When multiple subjects appear at the same indentation level, followed by a closing annotation at that same level, they should be parsed as a single verbatim group with multiple pairs.

## Actual Behavior

The parser produces separate sessions/definitions instead of a single verbatim group.

## Example

### File: `docs/specs/v1/elements/verbatim/verbatim-13-group-spades.lex`

```
This is a groupped Verbatim Block, this is the first Group: 
 $ pwd # always te staring point 
Now that you know where you are, lets find out what's around you: 

 $ ls
 $ ls -r # recursive

And let's go places: 
 $ cd <path to go>

Feeling lost, let's get back home: 
 $ cd ~
:: shell ::

Note that verbatim blocks conetents can have any number of blank lines, including None.
```

**Expected AST**: 2 items `[VerbatimBlock (with 4 groups), Paragraph]`

**Actual AST**: 6 items `[Session, Session, Session, Session, Annotation, Paragraph]`

## What Works

### File: `docs/specs/v1/elements/verbatim/verbatim-11-group-shell.lex`

This works correctly because each subject is immediately followed by a blank line:

```
Installing with home brew is simple:

    $ brew install lex
From there the interactive help is available:

    $ lex help
And the built-in viewer can be used to quickly view the parsing:

    $ lexv <path>
:: shell ::
```

**Pattern**: `Subject → BLANK → Content → Subject → BLANK → Content → ...`

**Why it works**: Each subject is immediately followed by a blank line, making the pattern unambiguous.

## What Fails

### File: `docs/specs/v1/elements/verbatim/verbatim-13-group-spades.lex`

**Pattern**: `Subject → Content (NO BLANK) → Subject → BLANK → Content → ...`

**Why it fails**:

- First subject has NO blank line after it
- Blank lines appear BETWEEN pairs, not immediately after subjects
- The parser doesn't correctly recognize that subjects at the same level after blank lines are continuing the verbatim group

## Root Cause Analysis

The verbatim block parser uses:

```rust
subject_content_pair.repeated().at_least(1).then(closing_annotation_parser)
```

Where `subject_content_pair` is:

```
subject → optional blank → content → optional blank
```

**The Problem**: The repetition logic may fail when:

1. There are blank lines between pairs (after content ends with Dedent, before next subject)
2. The first subject has no blank line after it (content starts immediately)
3. The parser may stop after the first pair instead of recognizing subsequent subjects

**Possible causes**:

1. **Content boundary detection**: After content ends with `Dedent`, blank lines, then another subject, the parser may not recognize this as continuing the verbatim group
2. **Repetition logic**: The `subject_content_pair.repeated()` may stop prematurely when it encounters blank lines between pairs
3. **Token consumption**: Blank lines between pairs may be consumed in a way that prevents the repetition from continuing

## Technical Details

### Parse Order (Correct)

The reference parser tries elements in this order:

1. `verbatim_block` - **FIRST** (correct priority - should win over sessions)
2. `annotation_parser`
3. `list_parser`
4. `definition_parser`
5. `session_parser` - **Catch-all** (tried last)
6. `paragraph`

**Key Principle**: If something could be both a verbatim block and a session, verbatim blocks win because they're more specific and tried first.

### Parser Code Location

- Parser: `src/lex/parsing/reference/builders.rs:395-509`
- Parse order: `src/lex/parsing/reference/parser.rs:59-66`
- Tests: `tests/elements_verbatim.rs` (lines 272-310, marked `#[ignore]`)

### Token Stream Analysis

For `verbatim-13-group-spades.lex`, the token stream should be:

1. Subject tokens (column 0) + Colon
2. (No blank line) → Indent token
3. Content tokens (tab-indented)
4. Dedent token (back to column 0)
5. Blank line token(s)
6. Another subject token (column 0) + Colon ← **Should continue the group**
7. Blank line token
8. Indent token
9. Content tokens
10. ... and so on until closing annotation

The parser should recognize that steps 6-9 represent another pair in the same verbatim group.

## Impact

- Tests are marked with `#[ignore]` for `test_verbatim_13_group_spades` and `test_verbatim_12_document_simple`
- Verbatim groups with blank lines between pairs don't work correctly
- Verbatim groups nested within sessions don't work correctly
- Core verbatim group functionality works for the common case (subjects with blank lines after them)

## Proposed Solution

1. **Fix repetition logic**: Ensure `subject_content_pair.repeated()` correctly handles blank lines between pairs
2. **Blank line handling**: The key issue is that blank lines between pairs need to be consumed by the NEXT pair (before its subject), not by the previous pair (after its content). This allows the repetition to continue correctly.
3. **Parser structure**: Split the parser into:
   - `first_pair`: Matches the first subject/content pair (no blank lines before subject)
   - `subsequent_pair`: Matches additional pairs (blank lines allowed before subject)
   - Both should NOT consume blank lines after content - let the next pair or closing annotation handle them
4. **Greedy matching**: The repetition should be greedy - continue matching subject/content pairs until finding a closing annotation at the subject indentation level

## Root Cause Analysis

**RESOLVED**: The issue was that when a subject line has a space after the colon (e.g., "Subject: \n"), the token stream contains:

1. Colon token
2. Whitespace token (the space)
3. BlankLine token (the newline)

The `subject_token_parser` was only looking for an optional BlankLine immediately after the Colon, but wasn't accounting for Whitespace tokens in between. This caused the parser to fail when it encountered the Whitespace token.

## Solution

Modified `subject_token_parser` in `src/lex/parsing/reference/builders.rs` to also ignore whitespace tokens after the colon before the optional blank line:

```rust
.then_ignore(
    // Ignore whitespace after colon (common formatting: "Subject: \n")
    filter(|(t, _): &TokenLocation| matches!(t, Token::Whitespace))
        .repeated()
        .then(blank_line.ignored().or_not())
        .ignored()
);
```

This allows the parser to correctly handle:

- "Subject:\n" (no space, no blank line after colon)
- "Subject: \n" (space after colon, then newline)
- "Subject:\n\n" (no space, blank line after colon)
- "Subject: \n\n" (space after colon, then blank line)

## Status

✅ **FIXED** - The parser now correctly handles verbatim groups with blank lines between pairs, even when there are spaces after colons.

- `test_verbatim_13_group_spades` now passes
- All other verbatim tests continue to pass
- `test_verbatim_12_document_simple` has a different issue (sessions not parsing correctly) and is unrelated to this fix

## Test Cases

- ✅ `verbatim-11-group-shell.lex` - Works (subjects with blank lines after them)
- ✅ `verbatim-13-group-spades.lex` - **FIXED** (blank lines between pairs, spaces after colons)
- ⚠️ `verbatim-12-document-simple.lex` - Has a different issue (sessions not parsing correctly, unrelated to this fix)

## Steps to Reproduce

1. Parse `docs/specs/v1/elements/verbatim/verbatim-13-group-spades.lex`
2. Expected: Single verbatim block with 4 groups
3. Actual: 4 separate sessions + annotation + paragraph

## Related

- PR #207: Initial verbatim groups implementation
- Task: `local/tasks/61-verbatim-groups-iteration.lex`
