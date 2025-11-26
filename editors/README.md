# Editor Tooling

As part of lex's value proposition, we'll be building two high quality edtior plugins for VSCode and Neovim.

While not entirely possible, we will keep feature parity between both and share as much code as possible. This unfolds as a few guiding principles:

1. Editor specific codee is only about the interactoin model: how a command is requested, how a buffer behaves. But all logic must be outside of the plugin code.  
2. That shared logic is to reside in the rust codebase, in the various crates. In common they will all be channeled but the LSP execute command protocol, which allows arbitrary commands to be invoked from lsp clientes, which both plugins are.
3. Whatever can be addressed by the regular LSP calls (sematic tokens, hover, symbols) should be so, and the execute command reserved for non standard operiations.
4. As much as possible, we want to mirror the aspects that can be configured by users, regardless of the configuration having diffrent forms.
5. We have a two  prong automated testing model: unit tests for the logic in the rust code, and shallow integration tests , e2e running on top of the actual editors and the plugins. Thes should only test the integration (things get called and returned and processed as expected, not testing many inputs and variants).

We will fully develop the initial version of plugins for Neovim and VSCode. While in the future these will be best served by dedicated repos, for now, as we are iterating over various layers in binaries, libs and the plugin themselves, they'll be colocated in the master lex repo.  

The design's goal is to have all logic-heavy lifting done in common rust code, and the plugins themselves being thin wrappers for each editors ui / entry points and interaction models.

Bellow the work in progress to be done, and at the documents very end the work already done.

## 2\. Document Handling

Being a markup document format, there is a set of features that are table stakes:

1. Interop:
2. Content Management:

## 3\. Editing

While the minimal featureset covered some of these, we have left out a few useful features related to editing:

1. Completion for paths, urls, citations.
2. Inserting Images / Files.
3. Inserting Verbatim Blocks from files.
4. Annotation management:
    1. Iterating through annotations
    2. Resolving, unresolving.
    3. Show Hide Annotations
5. Indenting on paste.
6. Tab shifting.
7. Ordering lists: fix the list orderings (even for nested lists)

## 4\. Publishing

While the interop features will generate various formats, the publishing workflow can be more detailed, alowing template selection, image sizing, previews, and so on.

This will be a core part of Lex, as it's value proposition is a single format from note to publication.

## 6\. Help / Documentation

Being a novel format, it would be very welcome to be able to offer help and documetation for the format itself. I'm not very sure how to best achieve this, but it's worth carving out the mental model for this.

## 7\. Shared Architecture

To avoid duplicating logic across plugins, we use the LSP `workspace/executeCommand` capability. This allows plugins to delegate complex tasks to the `lex-lsp` server.

**Mechanism**:

The server exposes a set of commands (e.g., `lex.echo`). Plugins invoke these commands using their editor's LSP client API.

**Usage**:

**VSCode**:

Use `vscode.commands.executeCommand('lex.commandName', args)`.

**Neovim**:

Use `client:exec_cmd({ command = 'lex.commandName', arguments = args })` (or `vim.lsp.buf.execute_command` for older versions).

## 8\. Finished Feature Packs

Bellow are the feature packs already implemented and live.

## 8\.1\. Minimal Featureset : Syntax and Language Support

These are the initial launch features for both editors:

1. Syntax Highlighting
2. Document Symbols
3. Hover Information
4. Folding Ranges
5. Formatting
6. Comment / Uncomment
7. Symbol Navigation (mostly references in Lex's context)

While diagnostics surely would make sense, given's Lex modus operandi of "never fails" and worst case scenario "parse as paragraphs", we don't have a useful , working implementation that would offer any value. This will be tackled later, but it is to be left out for now.

These are currently working in the Neovim plugin. All of these are built over the LSP protocol, with the lex-lsp binary being the server.

## 9\. Configuration

``` doc.table
| Option | Type | Default | Description |
|---|---|---|---|
| `session_blank_lines_before` | Integer | `1` | Number of blank lines before a session title. |
| `session_blank_lines_after` | Integer | `1` | Number of blank lines after a session title. |
| `normalize_seq_markers` | Boolean | `true` | Whether to normalize list markers (e.g. all bullets to `-`). |
| `unordered_seq_marker` | Character | `-` | The character to use for unordered list markers. |
| `max_blank_lines` | Integer | `2` | Maximum number of consecutive blank lines allowed. |
| `indent_string` | String | `"    "` | String to use for indentation (usually 4 spaces). |
| `preserve_trailing_blanks` | Boolean | `false` | Whether to preserve trailing blank lines at the end of the document. |
| `normalize_verbatim_markers` | Boolean | `true` | Whether to normalize verbatim markers to `::`.
```
