Lex Desktop Architecture
    
    This document outlines the architecture of the Lex Desktop application, the integration with the Rust-based Language Server Protocol (LSP), and the current state of syntax highlighting.

1. Architecture Overview

    The Lex Desktop application is built using a modern web technology stack wrapped in Electron, communicating with a high-performance Rust backend.

    1. Frontend (Renderer Process)
        
        Built with React, TypeScript, and Vite.
        
        Uses `monaco-editor` for the core editing experience.
        
        Manages the UI state, file handling via IPC, and renders the editor, sidebar, and panels.

    2. Backend (Main Process)
        
        Built with Electron (Node.js).
        
        Manages application lifecycle, native file system access, and window management.
        
        Spawns and manages the `lex-lsp` binary as a child process.
        
        Acts as a bridge between the Renderer and the LSP, forwarding JSON-RPC messages.

    3. Language Server (Rust)
        
        The `lex-lsp` binary, written in Rust.
        
        Provides language intelligence: diagnostics, document symbols (outline), hover information, and semantic tokens.
        
        Communicates via standard input/output (stdio) using the Language Server Protocol.

2. LSP Communications

    The communication between the Monaco editor and the Rust LSP server involves a multi-hop message passing system.

    1. Renderer to Main
        
        The `LspClient` in the renderer sends a JSON-RPC message (e.g., `textDocument/didChange`) via Electron IPC channel `lsp-input`.

    2. Main to LSP
        
        The Main process listens on `lsp-input`, prepends the required `Content-Length` header, and writes the data to the `lex-lsp` process's standard input (stdin).

    3. LSP Processing
        
        The `lex-lsp` binary reads from stdin, parses the message, processes the request (e.g., parsing the Lex document, generating tokens), and writes the response to standard output (stdout).

    4. LSP to Main
        
        The Main process captures the `lex-lsp` stdout stream.

    5. Main to Renderer
        
        The Main process forwards the raw data buffer to the renderer via Electron IPC channel `lsp-output`.

    6. Renderer Processing
        
        The `LspClient` buffers the incoming data, parses the `Content-Length` header, extracts the JSON body, and resolves the pending request or triggers a notification handler.

3. Syntax Highlighting

    Syntax highlighting in Lex Desktop is a hybrid system designed to provide immediate feedback while aiming for rich semantic understanding.

    1. The Goal: Semantic Highlighting
        
        The objective is to use the LSP's `textDocument/semanticTokens/full` capability to drive highlighting.
        
        The LSP server provides a detailed legend (token types like `SessionTitleText`, `ListMarker`) and returns encoded token arrays.
        
        Monaco's `DocumentSemanticTokensProvider` is designed to consume these tokens and apply themes.

    2. The Problem: Dormant Trigger
        
        Despite correct configuration, the Monaco editor in the current Electron/Vite environment fails to automatically trigger the `provideDocumentSemanticTokens` method of the registered provider.
        
        Manual invocation works, and the provider is correctly registered, but the automatic "spark" is missing.

    3. The Solution: Hybrid Fallback
        
        To ensure a functional user experience, a robust fallback system is implemented.

        1. Dynamic Registration (The Plumbing)
            
            The `Editor.tsx` component waits for the LSP to initialize.
            
            It extracts the *actual* semantic token legend from the server's capabilities.
            
            It registers the `DocumentSemanticTokensProvider` dynamically using this legend.
            
            If the server is already initialized (e.g., after a reload), it falls back to a static, compatible legend to ensure registration always succeeds.

        2. Monarch Fallback (The Visuals)
            
            A `MonarchTokensProvider` is registered alongside the semantic provider.
            
            This uses regular expressions to match core Lex elements:
            - Session Titles (`^Session Title.*`)
            - Comments (`^#.*`)
            - Verbatim Blocks
            
            The theme (`lex-theme`) defines rules for these Monarch tokens that match the intended design.

    4. Visual Style
        
        The editor uses the **Lex Monochrome Theme** (Dark Mode).
        
        Instead of colorful syntax highlighting, it relies on grayscale intensity and typography:
        - **Bold White**: Session Titles, Strong text.
        - **Italic Gray**: Markers, definitions.
        - **Underline**: References.
        
        This aligns with the design philosophy of Lex as a distraction-free, prose-focused format.
