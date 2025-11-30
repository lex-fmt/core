# Lex LSP Commands

This document lists the commands supported by the Lex Language Server (`lex-lsp`). These commands can be executed via the LSP `workspace/executeCommand` method.

## Commands

### `lex.echo`
**Title:** Echo
**Category:** Lex
**Description:** Echo back the input arguments. Useful for testing connection.

### `lex.import`
**Title:** Import Document
**Category:** Lex
**Description:** Import a document from another format (e.g., Markdown) into Lex format.

### `lex.export`
**Title:** Export Document
**Category:** Lex
**Description:** Export the current Lex document to another format (e.g., HTML, PDF, Markdown).

### `lex.next_annotation`
**Title:** Next Annotation
**Category:** Lex
**Description:** Navigate to the next annotation in the document.

### `lex.resolve_annotation`
**Title:** Resolve Annotation
**Category:** Lex
**Description:** Mark the annotation at the current cursor position as resolved.

### `lex.toggle_annotations`
**Title:** Toggle Annotations
**Category:** Lex
**Description:** Toggle the resolution status of the annotation at the current cursor position.

### `lex.insert_asset`
**Title:** Insert Asset
**Category:** Lex
**Description:** Insert a reference to an external asset (file) at the current cursor position.

### `lex.insert_verbatim`
**Title:** Insert Verbatim Block
**Category:** Lex
**Description:** Insert a new verbatim block with a specified label at the current cursor position.
