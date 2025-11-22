use crate::features::inline::{extract_inline_spans, InlineSpanKind};
use lex_parser::lex::ast::{
    Annotation, ContentItem, Definition, Document, List, ListItem, Paragraph, Range, Session,
    TextContent, Verbatim,
};
use lex_parser::lex::inlines::ReferenceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexSemanticTokenKind {
    SessionTitle,
    DefinitionSubject,
    ListMarker,
    AnnotationLabel,
    AnnotationParameter,
    InlineStrong,
    InlineEmphasis,
    InlineCode,
    InlineMath,
    Reference,
    ReferenceCitation,
    ReferenceFootnote,
    VerbatimSubject,
    VerbatimLanguage,
    VerbatimAttribute,
}

impl LexSemanticTokenKind {
    /// Returns the semantic token type string for LSP.
    ///
    /// We use standard LSP/markup token types where possible to ensure
    /// compatibility with existing editor themes (Neovim, VSCode, etc.)
    /// based on the Lex↔Markdown mapping from lex-babel.
    ///
    /// Mapping rationale (see lex-babel/src/formats/markdown/mod.rs):
    /// - Session → Heading → "markup.heading"
    /// - Definition → **Term**: Desc → "markup.bold" (for term)
    /// - InlineStrong → **bold** → "markup.bold"
    /// - InlineEmphasis → *italic* → "markup.italic"
    /// - InlineCode → `code` → "string" (standard for inline code)
    /// - InlineMath → $math$ → "number" (visually distinct, standard type)
    /// - Reference → [citation] → "markup.underline" (references are underlined)
    /// - Verbatim → ```block``` → "string" (code blocks)
    /// - Annotation → <!-- comment --> → "comment"
    /// - ListMarker → - or 1. → "operator" (punctuation-like)
    pub fn as_str(self) -> &'static str {
        match self {
            LexSemanticTokenKind::SessionTitle => "markup.heading",
            LexSemanticTokenKind::DefinitionSubject => "markup.bold",
            LexSemanticTokenKind::ListMarker => "operator",
            LexSemanticTokenKind::AnnotationLabel => "comment",
            LexSemanticTokenKind::AnnotationParameter => "parameter",
            LexSemanticTokenKind::InlineStrong => "markup.bold",
            LexSemanticTokenKind::InlineEmphasis => "markup.italic",
            LexSemanticTokenKind::InlineCode => "string",
            LexSemanticTokenKind::InlineMath => "number",
            LexSemanticTokenKind::Reference => "markup.underline",
            LexSemanticTokenKind::ReferenceCitation => "markup.underline",
            LexSemanticTokenKind::ReferenceFootnote => "markup.underline",
            LexSemanticTokenKind::VerbatimSubject => "string",
            LexSemanticTokenKind::VerbatimLanguage => "type",
            LexSemanticTokenKind::VerbatimAttribute => "parameter",
        }
    }
}

pub const SEMANTIC_TOKEN_KINDS: &[LexSemanticTokenKind] = &[
    LexSemanticTokenKind::SessionTitle,
    LexSemanticTokenKind::DefinitionSubject,
    LexSemanticTokenKind::ListMarker,
    LexSemanticTokenKind::AnnotationLabel,
    LexSemanticTokenKind::AnnotationParameter,
    LexSemanticTokenKind::InlineStrong,
    LexSemanticTokenKind::InlineEmphasis,
    LexSemanticTokenKind::InlineCode,
    LexSemanticTokenKind::InlineMath,
    LexSemanticTokenKind::Reference,
    LexSemanticTokenKind::ReferenceCitation,
    LexSemanticTokenKind::ReferenceFootnote,
    LexSemanticTokenKind::VerbatimSubject,
    LexSemanticTokenKind::VerbatimLanguage,
    LexSemanticTokenKind::VerbatimAttribute,
];

#[derive(Debug, Clone, PartialEq)]
pub struct LexSemanticToken {
    pub kind: LexSemanticTokenKind,
    pub range: Range,
}

pub fn collect_semantic_tokens(document: &Document) -> Vec<LexSemanticToken> {
    let mut collector = TokenCollector::new();
    collector.process_document(document);
    collector.finish()
}

struct TokenCollector {
    tokens: Vec<LexSemanticToken>,
}

impl TokenCollector {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn finish(mut self) -> Vec<LexSemanticToken> {
        self.tokens.sort_by(|a, b| {
            let a_start = (
                &a.range.start.line,
                &a.range.start.column,
                &a.range.end.line,
                &a.range.end.column,
            );
            let b_start = (
                &b.range.start.line,
                &b.range.start.column,
                &b.range.end.line,
                &b.range.end.column,
            );
            a_start.cmp(&b_start)
        });
        self.tokens
    }

    fn push_range(&mut self, range: &Range, kind: LexSemanticTokenKind) {
        if range.span.start < range.span.end {
            self.tokens.push(LexSemanticToken {
                kind,
                range: range.clone(),
            });
        }
    }

    fn process_document(&mut self, document: &Document) {
        self.process_annotations(document.annotations());
        self.process_session(&document.root, true);
    }

    fn process_session(&mut self, session: &Session, is_root: bool) {
        if !is_root {
            if let Some(header) = session.header_location() {
                self.push_range(header, LexSemanticTokenKind::SessionTitle);
            }
            self.process_text_content(&session.title);
        }

        self.process_annotations(session.annotations());
        for child in session.children.iter() {
            self.process_content_item(child);
        }
    }

    fn process_content_item(&mut self, item: &ContentItem) {
        match item {
            ContentItem::Paragraph(paragraph) => self.process_paragraph(paragraph),
            ContentItem::Session(session) => self.process_session(session, false),
            ContentItem::List(list) => self.process_list(list),
            ContentItem::ListItem(list_item) => self.process_list_item(list_item),
            ContentItem::Definition(definition) => self.process_definition(definition),
            ContentItem::Annotation(annotation) => self.process_annotation(annotation),
            ContentItem::VerbatimBlock(verbatim) => self.process_verbatim(verbatim),
            ContentItem::TextLine(text_line) => self.process_text_content(&text_line.content),
            ContentItem::VerbatimLine(_) => {}
            ContentItem::BlankLineGroup(_) => {}
        }
    }

    fn process_paragraph(&mut self, paragraph: &Paragraph) {
        for line in &paragraph.lines {
            if let ContentItem::TextLine(text_line) = line {
                self.process_text_content(&text_line.content);
            }
        }
        self.process_annotations(paragraph.annotations());
    }

    fn process_list(&mut self, list: &List) {
        self.process_annotations(list.annotations());
        for item in list.items.iter() {
            if let ContentItem::ListItem(list_item) = item {
                self.process_list_item(list_item);
            }
        }
    }

    fn process_list_item(&mut self, list_item: &ListItem) {
        if let Some(marker_range) = &list_item.marker.location {
            self.push_range(marker_range, LexSemanticTokenKind::ListMarker);
        }
        for text in &list_item.text {
            self.process_text_content(text);
        }
        self.process_annotations(list_item.annotations());
        for child in list_item.children.iter() {
            self.process_content_item(child);
        }
    }

    fn process_definition(&mut self, definition: &Definition) {
        if let Some(header) = definition.header_location() {
            self.push_range(header, LexSemanticTokenKind::DefinitionSubject);
        }
        self.process_text_content(&definition.subject);
        self.process_annotations(definition.annotations());
        for child in definition.children.iter() {
            self.process_content_item(child);
        }
    }

    fn process_verbatim(&mut self, verbatim: &Verbatim) {
        for group in verbatim.group() {
            self.process_text_content(group.subject);
            if let Some(location) = &group.subject.location {
                self.push_range(location, LexSemanticTokenKind::VerbatimSubject);
            }
        }

        self.push_range(
            &verbatim.closing_data.label.location,
            LexSemanticTokenKind::VerbatimLanguage,
        );
        for parameter in &verbatim.closing_data.parameters {
            self.push_range(&parameter.location, LexSemanticTokenKind::VerbatimAttribute);
        }

        self.process_annotations(verbatim.annotations());
    }

    fn process_annotation(&mut self, annotation: &Annotation) {
        self.push_range(
            annotation.header_location(),
            LexSemanticTokenKind::AnnotationLabel,
        );
        for parameter in &annotation.data.parameters {
            self.push_range(
                &parameter.location,
                LexSemanticTokenKind::AnnotationParameter,
            );
        }
        for child in annotation.children.iter() {
            self.process_content_item(child);
        }
    }

    fn process_annotations(&mut self, annotations: &[Annotation]) {
        for annotation in annotations {
            self.process_annotation(annotation);
        }
    }

    fn process_text_content(&mut self, text: &TextContent) {
        for span in extract_inline_spans(text) {
            let kind = match span.kind {
                InlineSpanKind::Strong => Some(LexSemanticTokenKind::InlineStrong),
                InlineSpanKind::Emphasis => Some(LexSemanticTokenKind::InlineEmphasis),
                InlineSpanKind::Code => Some(LexSemanticTokenKind::InlineCode),
                InlineSpanKind::Math => Some(LexSemanticTokenKind::InlineMath),
                InlineSpanKind::Reference(reference_type) => Some(match reference_type {
                    ReferenceType::Citation(_) => LexSemanticTokenKind::ReferenceCitation,
                    ReferenceType::FootnoteNumber { .. }
                    | ReferenceType::FootnoteLabeled { .. } => {
                        LexSemanticTokenKind::ReferenceFootnote
                    }
                    _ => LexSemanticTokenKind::Reference,
                }),
            };
            if let Some(kind) = kind {
                self.push_range(&span.range, kind);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::test_support::{sample_document, sample_source};
    use lex_parser::lex::testing::lexplore::Lexplore;

    fn snippets(
        tokens: &[LexSemanticToken],
        kind: LexSemanticTokenKind,
        source: &str,
    ) -> Vec<String> {
        tokens
            .iter()
            .filter(|token| token.kind == kind)
            .map(|token| source[token.range.span.clone()].to_string())
            .collect()
    }

    #[test]
    fn collects_structural_tokens() {
        let document = sample_document();
        let tokens = collect_semantic_tokens(&document);
        let source = sample_source();
        assert!(
            snippets(&tokens, LexSemanticTokenKind::SessionTitle, source)
                .iter()
                .any(|snippet| snippet.trim() == "1. Intro")
        );
        assert!(
            snippets(&tokens, LexSemanticTokenKind::DefinitionSubject, source)
                .iter()
                .any(|snippet| snippet.trim_end() == "Cache")
        );
        let markers = snippets(&tokens, LexSemanticTokenKind::ListMarker, source);
        assert_eq!(markers.len(), 4);
        assert!(markers
            .iter()
            .all(|snippet| snippet.trim_start().starts_with('-')
                || snippet.trim_start().chars().next().unwrap().is_numeric()));
        let annotation_labels = snippets(&tokens, LexSemanticTokenKind::AnnotationLabel, source);
        assert!(annotation_labels
            .iter()
            .any(|snippet| snippet.contains("doc.note")));
        let parameters = snippets(&tokens, LexSemanticTokenKind::AnnotationParameter, source);
        assert!(parameters
            .iter()
            .any(|snippet| snippet.contains("severity=info")));
        let verbatim_subjects = snippets(&tokens, LexSemanticTokenKind::VerbatimSubject, source);
        assert!(verbatim_subjects
            .iter()
            .any(|snippet| snippet.contains("CLI Example")));
        assert!(
            snippets(&tokens, LexSemanticTokenKind::VerbatimLanguage, source)
                .iter()
                .any(|snippet| snippet.contains("shell"))
        );
    }

    #[test]
    fn collects_inline_tokens() {
        let document = sample_document();
        let tokens = collect_semantic_tokens(&document);
        let source = sample_source();
        assert!(
            snippets(&tokens, LexSemanticTokenKind::InlineStrong, source)
                .iter()
                .any(|snippet| snippet.contains("Lex"))
        );
        assert!(
            snippets(&tokens, LexSemanticTokenKind::InlineEmphasis, source)
                .iter()
                .any(|snippet| snippet.contains("format"))
        );
        assert!(snippets(&tokens, LexSemanticTokenKind::InlineCode, source)
            .iter()
            .any(|snippet| snippet.contains("code")));
        assert!(snippets(&tokens, LexSemanticTokenKind::InlineMath, source)
            .iter()
            .any(|snippet| snippet.contains("math")));
    }

    #[test]
    fn classifies_references() {
        let document = sample_document();
        let tokens = collect_semantic_tokens(&document);
        let source = sample_source();
        assert!(
            snippets(&tokens, LexSemanticTokenKind::ReferenceCitation, source)
                .iter()
                .any(|snippet| snippet.contains("@spec2025"))
        );
        assert!(
            snippets(&tokens, LexSemanticTokenKind::ReferenceFootnote, source)
                .iter()
                .any(|snippet| snippet.contains("^source"))
        );
        assert!(
            snippets(&tokens, LexSemanticTokenKind::ReferenceFootnote, source)
                .iter()
                .any(|snippet| snippet.contains("1"))
        );
        assert!(snippets(&tokens, LexSemanticTokenKind::Reference, source)
            .iter()
            .any(|snippet| snippet.contains("Cache")));
    }

    #[test]
    fn empty_document_has_no_tokens() {
        let document = Lexplore::benchmark(0)
            .parse()
            .expect("failed to parse empty benchmark fixture");
        let tokens = collect_semantic_tokens(&document);
        assert!(tokens.is_empty());
    }
}
