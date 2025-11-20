LineTreeViz Format Implementation

## Goal

Create a new tree visualization format that provides 1:1 correspondence between source lines and output lines, collapsing container nodes (Paragraph, List) with their children while preserving structural context.

## Current State

- **treeviz**: AST-focused, shows all nodes including containers
- **domtreeviz**: Source-focused but uses hardcoded lookups for node categorization

## Problem

Node categorization (is_visual_line, is_meaningful_block) is hardcoded in formatter, but these are really AST structural properties.

## Solution

1. Add VisualStructure trait to AST with three properties:
   - is_source_line_node(): Nodes that appear as lines in source
   - has_visual_header(): Nodes with separate header lines (Session title, Definition subject)
   - collapses_with_children(): Homogeneous containers (Paragraph/TextLine, List/ListItem)

2. Implement trait on all AST node types

3. Create linetreeviz format that:
   - Iterates source lines
   - Uses AST traits instead of hardcoded lookups
   - Collapses containers by showing parent icon before child icon (¶ ↵, ☰ •)
   - Shares icon mapping with treeviz (extract to common module)

4. Replace domtreeviz with linetreeviz

## Implementation Steps

1. Add VisualStructure trait to lex-parser/src/lex/ast/traits.rs
2. Implement trait on: TextLine, Paragraph, ListItem, List, Session, Definition, Annotation, VerbatimBlock, VerbatimLine, BlankLineGroup
3. Extract icon mapping to shared module
4. Create lex-babel/src/formats/linetreeviz/mod.rs
5. Update format registry
6. Add tests
7. Remove domtreeviz
