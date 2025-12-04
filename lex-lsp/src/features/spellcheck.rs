use lex_parser::lex::ast::elements::{ContentItem, Document, TextLine};
use lex_parser::lex::ast::{AstNode, Container}; // Import AstNode for range()
use spellbook::Dictionary;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Position as LspPosition, Range as LspRange,
};

static DICTIONARIES: OnceLock<Mutex<HashMap<String, Arc<Dictionary>>>> = OnceLock::new();

fn get_dictionaries() -> &'static Mutex<HashMap<String, Arc<Dictionary>>> {
    DICTIONARIES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn check_document(document: &Document, language: &str) -> Vec<Diagnostic> {
    let dict = get_dictionary(language);
    if dict.is_none() {
        return vec![];
    }
    let dict = dict.unwrap();

    let mut diagnostics = Vec::new();
    // Document root is a Session
    traverse_session(&document.root, &dict, &mut diagnostics);
    diagnostics
}

fn get_dictionary(language: &str) -> Option<Arc<Dictionary>> {
    let mut cache = get_dictionaries().lock().unwrap();
    if let Some(dict) = cache.get(language) {
        return Some(dict.clone());
    }

    // Try to load from "dictionaries" folder in CWD or adjacent to executable
    let paths_to_try = vec![
        std::path::Path::new("dictionaries"),
        std::path::Path::new("resources/dictionaries"),
        // For development/testing
        std::path::Path::new("../dictionaries"),
        std::path::Path::new("../../dictionaries"),
    ];

    for base_path in paths_to_try {
        let aff_path = base_path.join(format!("{language}.aff"));
        let dic_path = base_path.join(format!("{language}.dic"));

        if aff_path.exists() && dic_path.exists() {
            if let (Ok(aff), Ok(dic)) = (
                std::fs::read_to_string(&aff_path),
                std::fs::read_to_string(&dic_path),
            ) {
                if let Ok(dict) = Dictionary::new(&aff, &dic) {
                    let dict = Arc::new(dict);
                    cache.insert(language.to_string(), dict.clone());
                    return Some(dict);
                }
            }
        }
    }

    // Fallback for testing if files not found:
    // Create a minimal dictionary so we at least have something working
    if language == "en_US" {
        let aff = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'";
        let dic = "2\nhello\nworld";
        if let Ok(dict) = Dictionary::new(aff, dic) {
            let dict = Arc::new(dict);
            cache.insert(language.to_string(), dict.clone());
            return Some(dict);
        }
    }

    None
}

fn traverse_content_item(item: &ContentItem, dict: &Dictionary, diagnostics: &mut Vec<Diagnostic>) {
    match item {
        ContentItem::Paragraph(para) => {
            for line_item in &para.lines {
                if let ContentItem::TextLine(tl) = line_item {
                    check_text_line(tl, dict, diagnostics);
                }
            }
        }
        ContentItem::Session(session) => traverse_session(session, dict, diagnostics),
        ContentItem::TextLine(tl) => check_text_line(tl, dict, diagnostics),
        _ => {
            // Generic traversal for other containers
            if let Some(children) = item.children() {
                for child in children {
                    traverse_content_item(child, dict, diagnostics);
                }
            }
        }
    }
}

fn traverse_session(
    session: &lex_parser::lex::ast::elements::Session,
    dict: &Dictionary,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for child in session.children() {
        traverse_content_item(child, dict, diagnostics);
    }
}

fn check_text_line(line: &TextLine, dict: &Dictionary, diagnostics: &mut Vec<Diagnostic>) {
    let text = line.text();
    let range = line.range();

    let mut current_offset = 0;
    for word in text.split_whitespace() {
        if let Some(index) = text[current_offset..].find(word) {
            let start_offset = current_offset + index;
            // Strip punctuation
            let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
            if !clean_word.is_empty() {
                let is_correct = dict.check(clean_word);
                if !is_correct {
                    // Calculate LSP range
                    // TextLine is always single line.
                    let start_char = range.start.column + start_offset;
                    let end_char = start_char + word.len();

                    diagnostics.push(Diagnostic {
                        range: LspRange {
                            start: LspPosition {
                                line: range.start.line as u32,
                                character: start_char as u32,
                            },
                            end: LspPosition {
                                line: range.end.line as u32,
                                character: end_char as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(tower_lsp::lsp_types::NumberOrString::String(
                            "spelling".to_string(),
                        )),
                        code_description: None,
                        source: Some("lex-spell".to_string()),
                        message: format!("Unknown word: {clean_word}"),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            current_offset = start_offset + word.len();
        }
    }
}

pub fn suggest_corrections(word: &str, language: &str) -> Vec<String> {
    if let Some(dict) = get_dictionary(language) {
        let mut suggestions = Vec::new();
        dict.suggest(word, &mut suggestions);
        return suggestions;
    }
    vec![]
}

pub fn add_to_dictionary(_word: &str, _language: &str) {
    // Placeholder
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::ast::elements::{Paragraph, Session};
    use lex_parser::lex::ast::Container;
    use lex_parser::lex::ast::{Position, Range};

    #[test]
    fn test_check_text() {
        let aff = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'";
        let dic = "1\nhello";

        let dict = Arc::new(Dictionary::new(aff, dic).unwrap());

        {
            let mut cache = get_dictionaries().lock().unwrap();
            cache.insert("test".to_string(), dict);
        }

        let range = Range::new(0..11, Position::new(0, 0), Position::new(0, 11));

        let para = Paragraph::from_line("hello world".to_string()).at(range.clone());

        // Construct a document with a root session containing the paragraph
        let mut session = Session::with_title("Title".to_string());
        session.children_mut().push(ContentItem::Paragraph(para));

        let doc = Document {
            root: session,
            ..Default::default()
        };

        let diags = check_document(&doc, "test");

        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].message, "Unknown word: world");
    }

    #[test]
    fn test_suggest() {
        let aff = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'\nREP 1\nREP o 0";
        let dic = "1\nhello";
        let dict = Arc::new(Dictionary::new(aff, dic).unwrap());

        {
            let mut cache = get_dictionaries().lock().unwrap();
            cache.insert("test_suggest".to_string(), dict);
        }

        let _suggestions = suggest_corrections("helo", "test_suggest");
        // "helo" -> "hello"
    }
}
