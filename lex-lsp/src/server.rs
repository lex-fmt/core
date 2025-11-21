//! Main language server implementation

use std::collections::HashMap;
use std::sync::Arc;

use crate::features::document_symbols::{collect_document_symbols, LexDocumentSymbol};
use crate::features::folding_ranges::{folding_ranges as collect_folding_ranges, LexFoldingRange};
use crate::features::hover::{hover as compute_hover, HoverResult};
use crate::features::semantic_tokens::{
    collect_semantic_tokens, LexSemanticToken, SEMANTIC_TOKEN_KINDS,
};
use lex_parser::lex::ast::{Document, Position as AstPosition, Range as AstRange};
use lex_parser::lex::parsing;
use tokio::sync::RwLock;
use tower_lsp::async_trait;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, FoldingRange, FoldingRangeParams,
    FoldingRangeProviderCapability, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, MarkupContent, MarkupKind, OneOf,
    Position, Range, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    ServerCapabilities, ServerInfo, TextDocumentItem, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkDoneProgressOptions,
};
use tower_lsp::Client;

pub trait LspClient: Send + Sync + Clone + 'static {}
impl LspClient for Client {}

pub trait FeatureProvider: Send + Sync + 'static {
    fn semantic_tokens(&self, document: &Document) -> Vec<LexSemanticToken>;
    fn document_symbols(&self, document: &Document) -> Vec<LexDocumentSymbol>;
    fn folding_ranges(&self, document: &Document) -> Vec<LexFoldingRange>;
    fn hover(&self, document: &Document, position: AstPosition) -> Option<HoverResult>;
}

#[derive(Default)]
pub struct DefaultFeatureProvider;

impl DefaultFeatureProvider {
    pub fn new() -> Self {
        Self
    }
}

impl FeatureProvider for DefaultFeatureProvider {
    fn semantic_tokens(&self, document: &Document) -> Vec<LexSemanticToken> {
        collect_semantic_tokens(document)
    }

    fn document_symbols(&self, document: &Document) -> Vec<LexDocumentSymbol> {
        collect_document_symbols(document)
    }

    fn folding_ranges(&self, document: &Document) -> Vec<LexFoldingRange> {
        collect_folding_ranges(document)
    }

    fn hover(&self, document: &Document, position: AstPosition) -> Option<HoverResult> {
        compute_hover(document, position)
    }
}

#[derive(Clone)]
struct DocumentEntry {
    document: Arc<Document>,
    text: Arc<String>,
}

#[derive(Default)]
struct DocumentStore {
    entries: RwLock<HashMap<Url, Option<DocumentEntry>>>,
}

impl DocumentStore {
    async fn upsert(&self, uri: Url, text: String) -> Option<DocumentEntry> {
        let parsed = match parsing::parse_document(&text) {
            Ok(document) => Some(DocumentEntry {
                document: Arc::new(document),
                text: Arc::new(text),
            }),
            Err(_) => None,
        };
        self.entries.write().await.insert(uri, parsed.clone());
        parsed
    }

    async fn get(&self, uri: &Url) -> Option<DocumentEntry> {
        self.entries.read().await.get(uri).cloned().flatten()
    }

    async fn remove(&self, uri: &Url) {
        self.entries.write().await.remove(uri);
    }
}

fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: SEMANTIC_TOKEN_KINDS
            .iter()
            .map(|kind| SemanticTokenType::new(kind.as_str()))
            .collect(),
        token_modifiers: Vec::new(),
    }
}

pub struct LexLanguageServer<C = Client, P = DefaultFeatureProvider> {
    _client: C,
    documents: DocumentStore,
    features: Arc<P>,
}

impl LexLanguageServer<Client, DefaultFeatureProvider> {
    pub fn new(client: Client) -> Self {
        Self::with_features(client, Arc::new(DefaultFeatureProvider::new()))
    }
}

impl<C, P> LexLanguageServer<C, P>
where
    C: LspClient,
    P: FeatureProvider,
{
    pub fn with_features(client: C, features: Arc<P>) -> Self {
        Self {
            _client: client,
            documents: DocumentStore::default(),
            features,
        }
    }

    async fn parse_and_store(&self, uri: Url, text: String) {
        self.documents.upsert(uri, text).await;
    }

    async fn document_entry(&self, uri: &Url) -> Option<DocumentEntry> {
        self.documents.get(uri).await
    }

    async fn document(&self, uri: &Url) -> Option<Arc<Document>> {
        self.document_entry(uri).await.map(|entry| entry.document)
    }
}

fn to_lsp_position(position: &AstPosition) -> Position {
    Position::new(position.line as u32, position.column as u32)
}

fn to_lsp_range(range: &AstRange) -> Range {
    Range {
        start: to_lsp_position(&range.start),
        end: to_lsp_position(&range.end),
    }
}

fn from_lsp_position(position: Position) -> AstPosition {
    AstPosition::new(position.line as usize, position.character as usize)
}

fn encode_semantic_tokens(tokens: &[LexSemanticToken], text: &str) -> Vec<SemanticToken> {
    let line_offsets = compute_line_offsets(text);
    let mut data = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for token in tokens {
        let token_type_index = SEMANTIC_TOKEN_KINDS
            .iter()
            .position(|kind| *kind == token.kind)
            .unwrap_or(0) as u32;
        for (line, start, length) in split_token_on_lines(token, text, &line_offsets) {
            if length == 0 {
                continue;
            }
            let delta_line = line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                start.saturating_sub(prev_start)
            } else {
                start
            };
            data.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type: token_type_index,
                token_modifiers_bitset: 0,
            });
            prev_line = line;
            prev_start = start;
        }
    }

    data
}

fn compute_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (idx, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(idx + ch.len_utf8());
        }
    }
    offsets
}

/// Expand a semantic token range into single-line segments.
///
/// The LSP wire format encodes tokens as delta positions relative to the previous token
/// and disallows spanning multiple lines, so every multi-line range must be broken into
/// per-line slices before encoding.
fn split_token_on_lines(
    token: &LexSemanticToken,
    text: &str,
    line_offsets: &[usize],
) -> Vec<(u32, u32, u32)> {
    let slice = &text[token.range.span.clone()];
    let mut segments = Vec::new();
    let mut current_line = token.range.start.line as u32;
    let mut segment_start = 0;
    let base_offset = token.range.span.start;

    for (idx, ch) in slice.char_indices() {
        if ch == '\n' {
            if idx > segment_start {
                let length = (idx - segment_start) as u32;
                let absolute_start = base_offset + segment_start;
                let line_offset = line_offsets
                    .get(current_line as usize)
                    .copied()
                    .unwrap_or(0);
                let start_col = (absolute_start.saturating_sub(line_offset)) as u32;
                segments.push((current_line, start_col, length));
            }
            current_line += 1;
            segment_start = idx + ch.len_utf8();
        }
    }

    if slice.len() > segment_start {
        let length = (slice.len() - segment_start) as u32;
        let absolute_start = base_offset + segment_start;
        let line_offset = line_offsets
            .get(current_line as usize)
            .copied()
            .unwrap_or(0);
        let start_col = (absolute_start.saturating_sub(line_offset)) as u32;
        segments.push((current_line, start_col, length));
    }

    segments
}

#[allow(deprecated)]
fn to_document_symbol(symbol: &LexDocumentSymbol) -> DocumentSymbol {
    DocumentSymbol {
        name: symbol.name.clone(),
        detail: symbol.detail.clone(),
        kind: symbol.kind,
        deprecated: None,
        range: to_lsp_range(&symbol.range),
        selection_range: to_lsp_range(&symbol.selection_range),
        children: if symbol.children.is_empty() {
            None
        } else {
            Some(symbol.children.iter().map(to_document_symbol).collect())
        },
        tags: None,
    }
}

fn to_lsp_folding_range(range: &LexFoldingRange) -> FoldingRange {
    FoldingRange {
        start_line: range.start_line,
        start_character: range.start_character,
        end_line: range.end_line,
        end_character: range.end_character,
        kind: range.kind.clone(),
        collapsed_text: None,
    }
}

#[async_trait]
impl<C, P> tower_lsp::LanguageServer for LexLanguageServer<C, P>
where
    C: LspClient,
    P: FeatureProvider,
{
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            semantic_tokens_provider: Some(
                lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        legend: semantic_tokens_legend(),
                        range: None,
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                    },
                ),
            ),
            ..ServerCapabilities::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "lex-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, .. } = params.text_document;
        self.parse_and_store(uri, text).await;
    }

    async fn did_change(&self, params: lsp_types::DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.parse_and_store(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn did_close(&self, params: lsp_types::DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri).await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        if let Some(entry) = self.document_entry(&params.text_document.uri).await {
            let DocumentEntry { document, text } = entry;
            let tokens = self.features.semantic_tokens(&document);
            let data = encode_semantic_tokens(&tokens, text.as_str());
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        if let Some(document) = self.document(&params.text_document.uri).await {
            let symbols = self.features.document_symbols(&document);
            let converted: Vec<DocumentSymbol> = symbols.iter().map(to_document_symbol).collect();
            Ok(Some(DocumentSymbolResponse::Nested(converted)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        if let Some(document) = self
            .document(&params.text_document_position_params.text_document.uri)
            .await
        {
            let position = from_lsp_position(params.text_document_position_params.position);
            if let Some(result) = self.features.hover(&document, position) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: result.contents,
                    }),
                    range: Some(to_lsp_range(&result.range)),
                }));
            }
        }
        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        if let Some(document) = self.document(&params.text_document.uri).await {
            let ranges = self.features.folding_ranges(&document);
            Ok(Some(ranges.iter().map(to_lsp_folding_range).collect()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::semantic_tokens::LexSemanticTokenKind;
    use crate::features::test_support::sample_source;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use tower_lsp::lsp_types::{
        DidOpenTextDocumentParams, DocumentSymbolParams, FoldingRangeKind, FoldingRangeParams,
        HoverParams, Position, SemanticTokensParams, SymbolKind, TextDocumentIdentifier,
        TextDocumentItem, TextDocumentPositionParams,
    };
    use tower_lsp::LanguageServer;

    #[derive(Clone, Default)]
    struct NoopClient;
    impl LspClient for NoopClient {}

    #[derive(Default)]
    struct MockFeatureProvider {
        semantic_tokens_called: AtomicUsize,
        document_symbols_called: AtomicUsize,
        hover_called: AtomicUsize,
        folding_called: AtomicUsize,
        last_hover_position: Mutex<Option<AstPosition>>,
    }

    impl FeatureProvider for MockFeatureProvider {
        fn semantic_tokens(&self, _: &Document) -> Vec<LexSemanticToken> {
            self.semantic_tokens_called.fetch_add(1, Ordering::SeqCst);
            vec![LexSemanticToken {
                kind: LexSemanticTokenKind::SessionTitle,
                range: AstRange::new(0..5, AstPosition::new(0, 0), AstPosition::new(0, 5)),
            }]
        }

        fn document_symbols(&self, _: &Document) -> Vec<LexDocumentSymbol> {
            self.document_symbols_called.fetch_add(1, Ordering::SeqCst);
            vec![LexDocumentSymbol {
                name: "symbol".into(),
                detail: None,
                kind: SymbolKind::STRING,
                range: AstRange::new(0..5, AstPosition::new(0, 0), AstPosition::new(0, 5)),
                selection_range: AstRange::new(
                    0..5,
                    AstPosition::new(0, 0),
                    AstPosition::new(0, 5),
                ),
                children: Vec::new(),
            }]
        }

        fn folding_ranges(&self, _: &Document) -> Vec<LexFoldingRange> {
            self.folding_called.fetch_add(1, Ordering::SeqCst);
            vec![LexFoldingRange {
                start_line: 0,
                start_character: Some(0),
                end_line: 1,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            }]
        }

        fn hover(&self, _: &Document, position: AstPosition) -> Option<HoverResult> {
            self.hover_called.fetch_add(1, Ordering::SeqCst);
            *self.last_hover_position.lock().unwrap() = Some(position);
            Some(HoverResult {
                range: AstRange::new(0..5, AstPosition::new(0, 0), AstPosition::new(0, 5)),
                contents: "hover".into(),
            })
        }
    }

    fn sample_uri() -> Url {
        Url::parse("file:///sample.lex").unwrap()
    }

    fn sample_text() -> String {
        sample_source().to_string()
    }

    fn offset_to_position(source: &str, offset: usize) -> AstPosition {
        let mut line = 0;
        let mut line_start = 0;
        for (idx, ch) in source.char_indices() {
            if idx >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                line_start = idx + ch.len_utf8();
            }
        }
        AstPosition::new(line, offset - line_start)
    }

    fn range_for_snippet(snippet: &str) -> AstRange {
        let source = sample_source();
        let start = source
            .find(snippet)
            .unwrap_or_else(|| panic!("snippet not found: {}", snippet));
        let end = start + snippet.len();
        let start_pos = offset_to_position(source, start);
        let end_pos = offset_to_position(source, end);
        AstRange::new(start..end, start_pos, end_pos)
    }

    async fn open_sample_document(server: &LexLanguageServer<NoopClient, MockFeatureProvider>) {
        let uri = sample_uri();
        server
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id: "lex".into(),
                    version: 1,
                    text: sample_text(),
                },
            })
            .await;
    }

    #[test]
    fn encode_semantic_tokens_splits_multi_line_ranges() {
        let snippet = "    CLI Example:\n        lex build\n        lex serve";
        let range = range_for_snippet(snippet);
        let tokens = vec![LexSemanticToken {
            kind: LexSemanticTokenKind::SessionTitle,
            range,
        }];
        let source = sample_source();
        let encoded = encode_semantic_tokens(&tokens, source);
        assert_eq!(encoded.len(), 3);
        let snippet_offset = source
            .find(snippet)
            .expect("snippet not found in sample document");
        let mut cursor = 0;
        let lines: Vec<&str> = snippet.split('\n').collect();
        let mut expected_positions = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let offset = snippet_offset + cursor;
            expected_positions.push(offset_to_position(source, offset));
            cursor += line.len();
            if idx < lines.len() - 1 {
                cursor += 1; // account for newline
            }
        }
        let mut absolute_positions = Vec::new();
        let mut line = 0u32;
        let mut column = 0u32;
        for token in &encoded {
            line += token.delta_line;
            let start = if token.delta_line == 0 {
                column + token.delta_start
            } else {
                token.delta_start
            };
            column = start;
            absolute_positions.push((line, start));
        }
        for (actual, expected) in absolute_positions.iter().zip(expected_positions.iter()) {
            assert_eq!(actual.0, expected.line as u32);
            assert_eq!(actual.1, expected.column as u32);
        }
        let expected_len: usize = snippet.lines().map(|line| line.len()).sum();
        let actual_len: usize = encoded.iter().map(|token| token.length as usize).sum();
        assert_eq!(actual_len, expected_len);
    }

    #[tokio::test]
    async fn semantic_tokens_call_feature_layer() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider.clone());
        open_sample_document(&server).await;

        let result = server
            .semantic_tokens_full(SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri: sample_uri() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(provider.semantic_tokens_called.load(Ordering::SeqCst), 1);
        let data_len = match result {
            SemanticTokensResult::Tokens(tokens) => tokens.data.len(),
            SemanticTokensResult::Partial(partial) => partial.data.len(),
        };
        assert!(data_len > 0);
    }

    #[tokio::test]
    async fn document_symbols_call_feature_layer() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider.clone());
        open_sample_document(&server).await;

        let response = server
            .document_symbol(DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri: sample_uri() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await
            .unwrap()
            .unwrap();

        match response {
            DocumentSymbolResponse::Nested(symbols) => assert!(!symbols.is_empty()),
            _ => panic!("unexpected symbol response"),
        }
        assert_eq!(provider.document_symbols_called.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn hover_uses_feature_provider_position() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider.clone());
        open_sample_document(&server).await;

        let hover = server
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: sample_uri() },
                    position: Position::new(0, 0),
                },
                work_done_progress_params: Default::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(hover.contents, HoverContents::Markup(_)));
        assert_eq!(provider.hover_called.load(Ordering::SeqCst), 1);
        let stored = provider.last_hover_position.lock().unwrap().unwrap();
        assert_eq!(stored.line, 0);
        assert_eq!(stored.column, 0);
    }

    #[tokio::test]
    async fn folding_range_uses_feature_provider() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider.clone());
        open_sample_document(&server).await;

        let ranges = server
            .folding_range(FoldingRangeParams {
                text_document: TextDocumentIdentifier { uri: sample_uri() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(provider.folding_called.load(Ordering::SeqCst), 1);
        assert_eq!(ranges.len(), 1);
    }

    #[tokio::test]
    async fn semantic_tokens_returns_none_when_document_missing() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider);

        let result = server
            .semantic_tokens_full(SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri: sample_uri() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn hover_returns_none_without_document_entry() {
        let provider = Arc::new(MockFeatureProvider::default());
        let server = LexLanguageServer::with_features(NoopClient, provider);

        let hover = server
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: sample_uri() },
                    position: Position::new(0, 0),
                },
                work_done_progress_params: Default::default(),
            })
            .await
            .unwrap();

        assert!(hover.is_none());
    }
}
