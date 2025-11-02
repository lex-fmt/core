# BlankLineGroup Conversion Plan

Goal: Make all elements stop consuming blank lines; let parent containers create BlankLineGroup nodes for all blank line sequences.

## Current State

✅ **Paragraph**: Already correct - stops at blank lines, standalone blanks become BlankLineGroup
✅ **Blank Line Positioning Verified**: Blank line at container level, not inside indented blocks

## Elements to Convert (Priority Order)

### 1. Session (Next Priority - Most Used)
**Current Pattern**: `[BLANK_LINE?] <TITLE_LINE> BLANK_LINE BLOCK`

**Issue**:
- Session consumes the blank line after the title (part of grammar.try_session_from_tree)
- This blank line should NOT be consumed by session itself
- Parent container should create a BlankLineGroup node for it

**Solution Approach**:
- Modify grammar.try_session_from_tree to NOT include the blank line in consumed count
- Session parser receives: `[BLANK_LINE?] <TITLE_LINE> BLANK_LINE BLOCK`
- Session parser creates: Session node (title + block content)
- Returns consumed = count up to but NOT including the blank line
- Parent loop creates BlankLineGroup node for the blank line after the session

**Test Case**:
```
Section:

    Content paragraph

Next element
```
Should parse as:
- Session("Section", [Paragraph("Content paragraph")])
- BlankLineGroup(1)
- Paragraph("Next element")

### 2. Definition
**Current Pattern**: `<SUBJECT>: <NEWLINE> BLOCK`

**Issue**: Definitions don't consume blank lines in their content (correct)
**But**: Definition content blank lines should become BlankLineGroup nodes

**Solution Approach**:
- Definition parsing is already correct
- Need to ensure walk_and_parse handles blanks in definition blocks correctly
- (This should already work via our BlankLineGroup conversion)

### 3. List/ListItem
**Current Pattern**: `LIST_MARKER ... [BLANK_LINE?] BLOCK ...`

**Issue**: List items can consume blank lines before nested blocks
**Solution Approach**:
- Similar to sessions - stop consuming blank lines
- Parent container (or list parsing) creates BlankLineGroup nodes

### 4. Annotation (Block Form)
**Current Pattern**: `:: <HEADER> :: <NEWLINE> BLOCK ... :: <FOOTER> ::`

**Issue**: Blank lines in content blocks should be preserved
**Solution Approach**:
- Content parsing already delegates to walk_and_parse
- Should work correctly once other elements are fixed

### 5. ForeignBlock
**Similar to Annotation**

## Implementation Order

1. **Session** - Most important, used frequently, affects many tests
2. **List/ListItem** - More complex, affects multiple places
3. **Definition** - Verify it already works correctly
4. **Annotation/ForeignBlock** - Should work once above are done

## Key Principle

**Elements should NOT consume blank lines that follow them.**
Blank lines are always handled at the parent/container level by creating BlankLineGroup nodes.

This keeps:
- Element parsing focused on their content
- Blank line handling centralized in walk_and_parse
- Separation of concerns clean
