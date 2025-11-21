//! Main language server implementation

use tower_lsp::Client;

/// The Lex Language Server
pub struct LexLanguageServer {
    #[allow(dead_code)]
    client: Client,
}

impl LexLanguageServer {
    /// Create a new Lex language server
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
