# Fix for Inline Reference Marker Highlighting in VSCode

## Problem

VSCode was not displaying syntax highlighting for inline reference markers (`[` and `]`) even though other inline markers (`*`, `_`, `` ` ``, `#`) were working correctly. The brackets were showing in blue (bracket colorization) instead of the configured gray color.

## Root Causes (Three Issues)

### Issue 1: Invalid Token Type IDs

VSCode semantic token type IDs must follow the pattern `letterOrDigit[-_letterOrDigit]*` (letters, digits, hyphens, and underscores only). The original token names used dots (`.`) which caused VSCode to silently reject them:

```
❌ InlineMarker.ref.start
❌ InlineMarker.strong.start
```

VSCode was showing validation errors in the test output:
```
[lex.lex-vscode]: 'configuration.semanticTokenType.id' must follow the pattern letterOrDigit[-_letterOrDigit]*
```

### Issue 2: Bracket Pair Colorization Override

Even after fixing the token IDs, `[` and `]` were registered as bracket pairs in `language-configuration.json`, causing VSCode's bracket pair colorization to override semantic token colors with blue.

### Issue 3: Overlapping Token Ranges in Definitions/Annotations

Reference markers inside definition bodies and annotation content were being overlapped by `DefinitionContent` and `AnnotationContent` tokens that spanned entire lines. When VSCode has overlapping tokens, it picks one styling - and the broader content tokens were winning over the specific inline markers.

Example:
```
Cache:
    A definition body referencing [Cache].
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DefinitionContent (spans entire line)
                                  ^     ^ InlineMarkerRefStart/End (overlapped!)
```

## Solution

### Part 1: Fix Token IDs

Changed all inline marker token IDs from dot-notation to underscore-notation:

```
✅ InlineMarker_ref_start
✅ InlineMarker_strong_start
```

### Part 2: Remove Brackets from Language Configuration

Removed `[` and `]` from the brackets array since they are inline reference markers in Lex, not structural brackets (unlike `{`, `}`, `(`, `)`)

### Part 3: Remove Overlapping Content Tokens

Removed emission of `DefinitionContent` and `AnnotationContent` tokens that spanned entire text lines, as they were overlapping with inline markers. The context is already clear from `DefinitionSubject` and `AnnotationLabel` tokens.

## Files Changed

1. **lex-analysis/src/semantic_tokens.rs:123-132** - Updated token name strings (dots → underscores)
2. **lex-analysis/src/semantic_tokens.rs:302-312** - Removed overlapping DefinitionContent/AnnotationContent tokens
3. **editors/vscode/package.json:127-165** - Updated semanticTokenTypes IDs (dots → underscores)
4. **editors/vscode/package.json:192-201** - Updated semanticTokenScopes mappings (dots → underscores)
5. **editors/vscode/themes/lex-light.json:135-142** - Added styling for `punctuation.definition.inline`
6. **editors/vscode/language-configuration.json:5-8** - Removed `["[", "]"]` from brackets array

## Testing

Created comprehensive test cases:

1. **test_inline_reference_markers.rs** - Verifies tokens for `[` and `]`, checks all reference types
2. **test_definition_reference_markers.rs** - Tests reference markers in definition bodies, detects overlapping ranges

All tests pass:
- ✅ Rust tests (lex-analysis): 28 passed
- ✅ VSCode integration tests: 10 passed
- ✅ No more validation errors in VSCode output
- ✅ No overlapping token ranges
