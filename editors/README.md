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

Being a markup document format, there is a set of features that are table stakes. Most of the
heavy lifting is already implemented in the workspace – editor integrations should focus on
discoverability and UX wrappers over those existing capabilities:

1. **Interop:** `lex-babel` ships conversions between Lex, Markdown, HTML and experimental PDF
   backends (see `lex-babel/src/formats`). Editors can either shell out to `lex-cli convert` or call
   future `workspace/executeCommand` endpoints that reuse Babel’s registry. No new parser logic is
   required inside the plugins.
2. **Content Management:** Document structure queries (sessions, annotations, definitions, links)
   live in `lex-analysis` and are already used by `lex-lsp` for document symbols, navigation and
   hover. Plugins should reuse those APIs via the LSP instead of keeping parallel indices.

## 3\. Editing

While the minimal featureset covered some of these, we have left out a few useful features related to
editing. The good news is that most of the primitives live in `lex-analysis`, so the new work is
mostly wiring them through LSP commands or standard requests:

1. Completion for paths, URLs, citations → build on top of `lex_parser::links` +
   `lex_analysis::reference_targets` once the LSP completion provider lands.
2. Inserting Images / Files.
3. Inserting Verbatim Blocks from files.
4. Annotation management:
    1. Iterating through annotations (reuse `lex_analysis::utils::find_annotation_at_position`).
    2. Resolving, unresolving via TextEdits that wrap formatter rules.
    3. Show/Hide annotations via editor UI toggles.
5. Indenting on paste → handled inside editor settings, but follow formatter defaults.
6. Tab shifting.
7. Ordering lists: fix the list ordering (even for nested lists) using formatter helpers already in
   `lex-babel`.

## 4\. Publishing

While the interop features generate the base artifacts, the publishing workflow can be more detailed,
allowing template selection, image sizing, previews, and so on. The current building blocks are:

1. `lex-babel` format registry → renders Markdown, HTML, LaTeX, and experimental PDF targets.
2. `lex-cli publish --format <target>` → batch entry point suitable for custom editor commands.
3. Planned LSP commands (see `editors/required-apis.lex`) → wrap Babel conversions and stream
   results back to editors for quick previews.

This will be a core part of Lex, as its value proposition is a single format from note to publication.

## 6\. Help / Documentation

Being a novel format, it is very welcome to offer help and documentation for the format itself. The
plan is to surface `docs/specs` and `specs/v*/general.lex` content through a generic `lex.help`
command so every editor can show contextual help without bundling duplicated assets.

Diagnostics are intentionally postponed. Lex is designed to never fail parsing and to fall back to
paragraphs no matter what, which makes traditional diagnostics less meaningful. We will revisit this
once we have a principled design for surfacing structural issues.

## 7\. Shared Architecture

To avoid duplicating logic across plugins, we keep all parsing, semantics, and conversions inside the
Rust crates and expose them through LSP features or thin CLI wrappers.

| Layer | Responsibilities | Crate(s) |
| --- | --- | --- |
| Parsing & AST | Turn text into structured documents, track ranges, resolve links | `lex-parser` |
| Analysis | Sessions, annotations, navigation, semantic tokens, folding | `lex-analysis` (re-exported by `lex-lsp`) |
| Formatting & Conversions | Canonical Lex serialization, Markdown/HTML/PDF transforms | `lex-babel` |
| Protocol adapters | LSP server, TextEdit diffs, document links, executeCommand bridge | `lex-lsp` |
| UX wrappers | Editor specific bindings (commands, menus, keymaps) | `editors/vscode`, `editors/nvim` |

Whenever a feature fits a standard LSP method (semantic tokens, hover, symbols, etc.) we rely on the
built-in request rather than inventing a command. For everything else we use the
`workspace/executeCommand` capability so plugins can delegate complex tasks to the `lex-lsp` server.

**Mechanism**:

The server exposes a set of commands (currently `lex.echo` while the real commands are under
development). Plugins invoke these commands using their editor's LSP client API.

**Usage**:

**VSCode**:

Use `vscode.commands.executeCommand('lex.commandName', args)`.

**Neovim**:

Use `client:exec_cmd({ command = 'lex.commandName', arguments = args })` (or `vim.lsp.buf.execute_command` for older versions).

## 8\. Finished Feature Packs

Bellow are the feature packs already implemented and live.

### 8.1. Minimal Featureset : Syntax and Language Support

These are the initial launch features for both editors:

1. Syntax Highlighting
2. Document Symbols
3. Hover Information
4. Folding Ranges
5. Formatting
6. Comment / Uncomment
7. Symbol Navigation (mostly references in Lex's context)

Diagnostics are intentionally postponed. Lex never fails parsing and always falls back to paragraphs,
so traditional error surfacing needs a more principled design than we currently have. We will revisit
the feature once we land on a meaningful signal we can expose to users.

These are currently working in the Neovim plugin. All of these are built over the LSP protocol, with the lex-lsp binary being the server.

## 9\. Configuration

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| \`session\_blank\_lines\_before\` | Integer | \`1\` | Number of blank lines before a session title. |
| \`session\_blank\_lines\_after\` | Integer | \`1\` | Number of blank lines after a session title. |
| \`normalize\_seq\_markers\` | Boolean | \`true\` | Whether to normalize list markers (e.g. all bullets to \`-\`). |
| \`unordered\_seq\_marker\` | Character | \`-\` | The character to use for unordered list markers. |
| \`max\_blank\_lines\` | Integer | \`2\` | Maximum number of consecutive blank lines allowed. |
| \`indent\_string\` | String | \`"    "\` | String to use for indentation (usually 4 spaces). |
| \`preserve\_trailing\_blanks\` | Boolean | \`false\` | Whether to preserve trailing blank lines at the end of the document. |
| \`normalize\_verbatim\_markers\` | Boolean | \`true\` | Whether to normalize verbatim markers to \`::\`. |
