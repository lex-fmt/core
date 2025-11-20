# AST Inspection and CLI UX Improvements

This PR significantly improves the `lex` CLI tool's usability and documentation, adding powerful new features for inspecting and visualizing the complete Abstract Syntax Tree (AST).

## Summary

- ✨ New `--extra-ast-full` parameter shows complete AST structure including all node properties
- 🎯 Boolean flag support for `--extra-*` parameters (no value required)
- 📚 Comprehensive CLI help documentation with examples
- 🔍 Extended tree visualization improvements
- 📖 Enhanced source code documentation

## Key Features

### 1. Complete AST Visualization with `--extra-ast-full`

Previously, AST visualizations only showed content structure. Now with `--extra-ast-full`, you can see **every** AST node property:

**Before** (default view):
```
⧉ Document (1 annotations, 0 items)
```

**After** (with `--extra-ast-full`):
```
⧉ Document (1 annotations, 0 items)
└─ " documentation
  ├─ ○ documentation                    # Annotation label
  ├─ ¶ This annotation documents...
  │ └─ ↵ This annotation documents...
  ├─ ≔ API
  │ ├─ ○ API                             # Definition subject
  │ ├─ ¶ Application Programming...
  │ │ └─ ↵ Application Programming...
```

**What's included:**
- Document-level annotations (with `"` icon)
- Session titles (as `SessionTitle` nodes)
- List item markers and text (as `Marker` and `Text` nodes)
- Definition subjects (as `Subject` nodes)
- Annotation labels and parameters (as `Label` and `Parameter` nodes)

### 2. Boolean Flag Support

Extra parameters can now be used as boolean flags:

```bash
# Old style (still works)
lex inspect file.lex --extra-ast-full true

# New style (cleaner)
lex inspect file.lex --extra-ast-full

# Boolean flags default to "true"
lex inspect file.lex ast-tag --extra-ast-full
```

### 3. Format Support

The `ast-full` parameter works with both visualization formats:

**Tree visualization:**
```bash
lex inspect file.lex ast-treeviz --extra-ast-full
```

**XML-like tags:**
```bash
lex inspect file.lex ast-tag --extra-ast-full
```

Output shows complete structure:
```xml
<document>
  <annotation>documentation
    <label>documentation</label>
    <paragraph>This annotation documents some terms.
      <text-line>This annotation documents some terms.</text-line>
    </paragraph>
    <definition>API
      <subject>API</subject>
      ...
```

### 4. Improved CLI Documentation

All commands now have comprehensive help with examples:

```bash
lex --help              # Overview with examples
lex inspect --help      # All transforms, stages, and extra params
lex convert --help      # All formats and usage patterns
```

**Main help includes:**
- Command overview
- Extra parameters explanation
- Usage examples for common tasks

**Inspect help shows:**
- All available transforms (ast-*, token-*, ir-*)
- Processing stages explanation
- Extra parameter documentation
- Examples for each transform type

**Convert help shows:**
- Supported formats (lex, markdown, html, tag)
- Auto-detection behavior
- Output options (stdout vs file)
- Conversion examples

## Technical Details

### Architecture Changes

**1. Snapshot System Enhancement** (`lex-parser/src/lex/ast/snapshot.rs`)
- Added `snapshot_from_document_with_options(doc, include_all)`
- Added `snapshot_from_content_with_options(item, include_all)`
- Updated all builder functions to accept `include_all` parameter
- When `include_all=true`, exposes all AST properties as child nodes

**2. Format Serializer Updates**
- **Treeviz** (`lex-babel/src/formats/treeviz/mod.rs`):
  - `to_treeviz_str_with_params(doc, params)` - accepts parameter map
  - Checks for `"ast-full"` parameter
- **Tag** (`lex-babel/src/formats/tag/mod.rs`):
  - `serialize_document_with_params(doc, params)` - accepts parameter map
  - Mirrors treeviz functionality

**3. CLI Parameter Parsing** (`lex-cli/src/main.rs`)
- Enhanced `parse_extra_args()` to detect boolean flags
- Distinguishes between `--extra-key value` and `--extra-key` (boolean)
- Boolean flags without values default to `"true"`

**4. Transform Pipeline** (`lex-cli/src/transforms.rs`)
- Updated `execute_transform()` to pass parameters through
- Wired `extra_params` to both tag and treeviz serializers

### Implementation Details

**Centralized AST Walking:**
All serializers (JSON, tag, treeviz) use the same snapshot system, ensuring:
- Consistent behavior across formats
- Single source of truth for AST traversal
- Easy addition of new parameters in the future

**Type-Safe Parameter Handling:**
Parameters flow through a typed HashMap, with format-specific parsing:
```rust
let include_all = params
    .get("ast-full")
    .map(|v| v.to_lowercase() == "true")
    .unwrap_or(false);
```

## Testing

All changes include comprehensive test coverage:

**CLI Tests:**
- Boolean flag parsing (single, multiple, mixed)
- Parameter extraction and passing
- Backward compatibility with explicit values

**Format Tests:**
- Tag format with ast-full parameter
- Treeviz format with ast-full parameter
- Annotation visibility in output

**Integration Tests:**
- All 750 tests pass
- No regressions in existing functionality

## Usage Examples

**Debugging parser behavior:**
```bash
# See what properties the parser extracted
lex inspect complex-doc.lex --extra-ast-full
```

**Understanding annotations:**
```bash
# See document-level and inline annotations
lex inspect annotated.lex ast-tag --extra-ast-full
```

**Verifying structure:**
```bash
# Confirm session titles, list markers, definition subjects
lex inspect structured.lex --extra-ast-full
```

**Learning the format:**
```bash
# Compare normal vs full view to understand AST structure
lex inspect example.lex > normal.txt
lex inspect example.lex --extra-ast-full > full.txt
diff normal.txt full.txt
```

## Documentation Improvements

**Module-level docs:**
- Added pipeline explanation to `transforms.rs`
- Documented all processing stages (tokenization → parsing → assembly)
- Explained transform naming convention (stage-format)

**Function-level docs:**
- Added comprehensive examples to `execute_transform()`
- Documented parameters for `to_treeviz_str_with_params()`
- Documented parameters for `serialize_document_with_params()`
- Included usage patterns and expected outputs

**CLI help text:**
- Added long_about to all commands
- Included examples in help output
- Explained extra parameters in context
- Listed supported formats and transforms

## Breaking Changes

None. All changes are backward compatible:
- Existing `--extra-key value` syntax still works
- Default behavior unchanged (ast-full is opt-in)
- All existing tests pass without modification

## Future Enhancements

This PR establishes the infrastructure for additional parameters:

**Potential future parameters:**
- `--extra-max-depth N` - Limit tree depth
- `--extra-compact` - Condensed output
- `--extra-show-locations` - Include line:column info
- `--extra-filter <type>` - Only show specific node types

The parameter system is extensible and format-agnostic.

## Commits

1. `feat(ast): expand ast-full to show all node properties` - Core snapshot system changes
2. `feat(cli): support ast-full for tag format and boolean flags` - CLI enhancements
3. `docs(cli): comprehensive documentation for commands and API` - Documentation improvements

---

## Checklist

- [x] All tests pass (750/750)
- [x] No breaking changes
- [x] Comprehensive documentation
- [x] CLI help updated
- [x] Source code documented
- [x] Examples provided
- [x] Pre-commit hooks pass
