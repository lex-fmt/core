use lex_parser::lex::ast::{
    Annotation, AstNode, ContentItem, Definition, Document, List, Range, Session, TextContent,
    Verbatim,
};
use lsp_types::SymbolKind;

#[derive(Debug, Clone, PartialEq)]
pub struct LexDocumentSymbol {
    pub name: String,
    pub detail: Option<String>,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<LexDocumentSymbol>,
}

pub fn collect_document_symbols(document: &Document) -> Vec<LexDocumentSymbol> {
    let mut symbols: Vec<LexDocumentSymbol> = document
        .annotations()
        .iter()
        .map(annotation_symbol)
        .collect();
    symbols.extend(session_symbols(&document.root, true));
    symbols
}

fn session_symbols(session: &Session, is_root: bool) -> Vec<LexDocumentSymbol> {
    let mut symbols = Vec::new();
    if !is_root {
        let mut children = annotation_symbol_list(session.annotations());
        children.extend(collect_symbols_from_items(session.children.iter()));
        let selection_range = session
            .header_location()
            .cloned()
            .unwrap_or_else(|| session.range().clone());
        symbols.push(LexDocumentSymbol {
            name: summarize_text(&session.title, "Session"),
            detail: Some(format!("{} item(s)", session.children.len())),
            kind: SymbolKind::NAMESPACE,
            range: session.range().clone(),
            selection_range,
            children,
        });
    } else {
        symbols.extend(collect_symbols_from_items(session.children.iter()));
    }
    symbols
}

fn collect_symbols_from_items<'a>(
    items: impl Iterator<Item = &'a ContentItem>,
) -> Vec<LexDocumentSymbol> {
    let mut symbols = Vec::new();
    for item in items {
        match item {
            ContentItem::Session(session) => symbols.extend(session_symbols(session, false)),
            ContentItem::Definition(definition) => symbols.push(definition_symbol(definition)),
            ContentItem::List(list) => symbols.push(list_symbol(list)),
            ContentItem::Annotation(annotation) => symbols.push(annotation_symbol(annotation)),
            ContentItem::VerbatimBlock(verbatim) => symbols.push(verbatim_symbol(verbatim)),
            ContentItem::Paragraph(paragraph) => {
                symbols.extend(annotation_symbol_list(paragraph.annotations()));
            }
            ContentItem::ListItem(list_item) => {
                symbols.extend(annotation_symbol_list(list_item.annotations()));
                symbols.extend(collect_symbols_from_items(list_item.children.iter()));
            }
            ContentItem::TextLine(_)
            | ContentItem::VerbatimLine(_)
            | ContentItem::BlankLineGroup(_) => {}
        }
    }
    symbols
}

fn definition_symbol(definition: &Definition) -> LexDocumentSymbol {
    let mut children = annotation_symbol_list(definition.annotations());
    children.extend(collect_symbols_from_items(definition.children.iter()));
    let selection_range = definition
        .header_location()
        .cloned()
        .unwrap_or_else(|| definition.range().clone());
    LexDocumentSymbol {
        name: summarize_text(&definition.subject, "Definition"),
        detail: Some("definition".to_string()),
        kind: SymbolKind::STRUCT,
        range: definition.range().clone(),
        selection_range,
        children,
    }
}

fn list_symbol(list: &List) -> LexDocumentSymbol {
    let mut children = annotation_symbol_list(list.annotations());
    for item in list.items.iter() {
        if let ContentItem::ListItem(list_item) = item {
            children.extend(annotation_symbol_list(list_item.annotations()));
            children.extend(collect_symbols_from_items(list_item.children.iter()));
        }
    }
    LexDocumentSymbol {
        name: format!("List ({} items)", list.items.len()),
        detail: None,
        kind: SymbolKind::ARRAY,
        range: list.range().clone(),
        selection_range: list.range().clone(),
        children,
    }
}

fn verbatim_symbol(verbatim: &Verbatim) -> LexDocumentSymbol {
    let children = annotation_symbol_list(verbatim.annotations());
    LexDocumentSymbol {
        name: format!(
            "Verbatim: {}",
            summarize_text(&verbatim.subject, "Verbatim block")
        ),
        detail: Some(verbatim.closing_data.label.value.clone()),
        kind: SymbolKind::OBJECT,
        range: verbatim.range().clone(),
        selection_range: verbatim
            .subject
            .location
            .clone()
            .unwrap_or_else(|| verbatim.range().clone()),
        children,
    }
}

fn annotation_symbol(annotation: &Annotation) -> LexDocumentSymbol {
    let children = collect_symbols_from_items(annotation.children.iter());
    LexDocumentSymbol {
        name: format!(":: {} ::", annotation.data.label.value),
        detail: if annotation.data.parameters.is_empty() {
            None
        } else {
            Some(
                annotation
                    .data
                    .parameters
                    .iter()
                    .map(|param| format!("{}={}", param.key, param.value))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        },
        kind: SymbolKind::EVENT,
        range: annotation.range().clone(),
        selection_range: annotation.header_location().clone(),
        children,
    }
}

fn annotation_symbol_list<'a>(
    annotations: impl IntoIterator<Item = &'a Annotation>,
) -> Vec<LexDocumentSymbol> {
    annotations.into_iter().map(annotation_symbol).collect()
}

fn summarize_text(text: &TextContent, fallback: &str) -> String {
    let trimmed = text.as_string().trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::test_support::sample_document;

    fn find_symbol<'a>(symbols: &'a [LexDocumentSymbol], name: &str) -> &'a LexDocumentSymbol {
        symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap_or_else(|| panic!("symbol {} not found", name))
    }

    #[test]
    fn builds_session_tree() {
        let document = sample_document();
        let symbols = collect_document_symbols(&document);
        assert!(symbols.iter().any(|s| s.name == ":: doc.note ::"));
        let session = find_symbol(&symbols, "1. Intro");
        let child_names: Vec<_> = session
            .children
            .iter()
            .map(|child| child.name.clone())
            .collect();
        assert!(child_names.iter().any(|name| name.contains("Cache")));
        assert!(child_names.iter().any(|name| name.contains("List")));
        assert!(child_names.iter().any(|name| name.contains("Verbatim")));
        let definition_symbol = session
            .children
            .iter()
            .find(|child| child.name.contains("Cache"))
            .expect("definition symbol not found");
        assert!(definition_symbol
            .children
            .iter()
            .any(|child| child.name.contains(":: callout ::")));
    }

    #[test]
    fn includes_document_level_annotations() {
        let document = sample_document();
        let symbols = collect_document_symbols(&document);
        assert!(symbols.iter().any(|symbol| symbol.name == ":: 42 ::"));
        assert!(symbols.iter().any(|symbol| symbol.name == ":: source ::"));
    }
}
