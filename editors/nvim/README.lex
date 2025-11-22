To open nvim with the right config and a lex doc loaded: 
  nvim -u editors/nvim/test/minimal_init.lua -c "e specs/v1/benchmark/20-ideas-naked.lex"
:: shell

  How to Interactively Verify Each Feature:

  1. Hover (textDocument/hover)

  - Move your cursor over any text (like section:, annotations @tag:, etc.)
  - Press K (shift+k) in normal mode
  - You should see a hover popup with information about that element

  2. Semantic Tokens (textDocument/semanticTokens/full)

  - Just open the file - semantic tokens provide syntax highlighting
  - Look for different colors on:
    - section: keywords (should be highlighted as session titles)
    - @tag: annotations
    - List markers -
    - Any inline formatting if present

  3. Document Symbols (textDocument/documentSymbol)

  - In normal mode, type: :lua vim.lsp.buf.document_symbol()
  - Or use the Telescope picker if available: :Telescope lsp_document_symbols
  - You should see an outline/hierarchy of:
    - Sections
    - Annotations
    - Lists
    - Definitions

  4. Folding Ranges (textDocument/foldingRange)

  - Move cursor to a section line (e.g., line 3: section: introduction)
  - Type zc to close/fold the section
  - Type zo to open/unfold the section
  - Type za to toggle fold
  - You should see the section content collapse/expand

  Additional Verification Commands:

  Check LSP is attached:
  :LspInfo

  View all available LSP capabilities:
  :lua print(vim.inspect(vim.lsp.get_active_clients()[1].server_capabilities))

  Manual LSP request for semantic tokens:
  :lua vim.lsp.buf_request(0, 'textDocument/semanticTokens/full', {textDocument =
  vim.lsp.util.make_text_document_params()}, function(err, result) print(vim.inspect(result)) 
  end)
