use lex_babel::transforms::serialize_to_lex;
use lex_parser::lex::ast::range::SourceLocation;
use lex_parser::lex::ast::Document;
use similar::{Algorithm, ChangeTag, TextDiff};

/// Text edit expressed as byte offsets over the original document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditSpan {
    pub start: usize,
    pub end: usize,
    pub new_text: String,
}

/// Inclusive/exclusive line range used for range formatting filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    fn clamp(self, line_count: usize) -> Self {
        let start = self.start.min(line_count);
        let mut end = self.end.min(line_count);
        if end <= start {
            end = (start + 1).min(line_count.max(1));
        }
        LineRange { start, end }
    }
}

/// Produce formatting edits for the entire document.
pub fn format_document(document: &Document, source: &str) -> Vec<TextEditSpan> {
    let formatted = match serialize_to_lex(document) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };
    compute_edits(source, &formatted)
}

/// Produce formatting edits limited to the provided line range.
pub fn format_range(document: &Document, source: &str, range: LineRange) -> Vec<TextEditSpan> {
    let formatted = match serialize_to_lex(document) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };
    let all_edits = compute_edits(source, &formatted);
    if all_edits.is_empty() {
        return all_edits;
    }
    let locator = SourceLocation::new(source);
    let clamped = range.clamp(locator.line_count());
    all_edits
        .into_iter()
        .filter(|span| edit_within_range(span, &locator, clamped))
        .collect()
}

fn compute_edits(original: &str, formatted: &str) -> Vec<TextEditSpan> {
    if original == formatted {
        return Vec::new();
    }

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(original, formatted);

    let line_offsets = compute_line_offsets(original);
    let mut edits = Vec::new();
    let mut builder: Option<EditBuilder> = None;
    let mut cursor = 0usize;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                cursor += 1;
                flush_builder(&mut builder, &mut edits);
            }
            ChangeTag::Delete => {
                let line = cursor;
                cursor += 1;
                let start = offset_for_line(line, &line_offsets, original.len());
                let end = offset_for_line(line + 1, &line_offsets, original.len());
                extend_delete(&mut builder, &mut edits, start, end);
            }
            ChangeTag::Insert => {
                let line = cursor;
                let offset = offset_for_line(line, &line_offsets, original.len());
                let text = change.value().to_string();
                if !text.is_empty() {
                    extend_insert(&mut builder, &mut edits, offset, text);
                }
            }
        }
    }

    flush_builder(&mut builder, &mut edits);
    edits
}

fn edit_within_range(span: &TextEditSpan, locator: &SourceLocation, range: LineRange) -> bool {
    if span.start == span.end {
        // Pure insertion
        let pos = locator.byte_to_position(span.start);
        return pos.line >= range.start && pos.line < range.end;
    }
    let start = locator.byte_to_position(span.start);
    let end = locator.byte_to_position(span.end);
    start.line >= range.start && end.line <= range.end
}

fn extend_delete(
    builder: &mut Option<EditBuilder>,
    edits: &mut Vec<TextEditSpan>,
    start: usize,
    end: usize,
) {
    match builder {
        Some(current) => {
            if start > current.end {
                flush_builder(builder, edits);
                *builder = Some(EditBuilder::new(start, end));
            } else if end > current.end {
                current.end = end;
            }
        }
        None => {
            *builder = Some(EditBuilder::new(start, end));
        }
    }
}

fn extend_insert(
    builder: &mut Option<EditBuilder>,
    edits: &mut Vec<TextEditSpan>,
    offset: usize,
    text: String,
) {
    match builder {
        Some(current) => {
            if offset < current.start || offset > current.end {
                flush_builder(builder, edits);
                *builder = Some(EditBuilder::at(offset));
            }
            if let Some(edit) = builder.as_mut() {
                if offset > edit.end {
                    edit.end = offset;
                }
                edit.new_text.push_str(&text);
            }
        }
        None => {
            let mut edit = EditBuilder::at(offset);
            edit.new_text.push_str(&text);
            *builder = Some(edit);
        }
    }
}

fn flush_builder(builder: &mut Option<EditBuilder>, edits: &mut Vec<TextEditSpan>) {
    if let Some(edit) = builder.take() {
        edits.push(edit.into_span());
    }
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

fn offset_for_line(line: usize, offsets: &[usize], text_len: usize) -> usize {
    offsets.get(line).copied().unwrap_or(text_len)
}

struct EditBuilder {
    start: usize,
    end: usize,
    new_text: String,
}

impl EditBuilder {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            new_text: String::new(),
        }
    }

    fn at(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    fn into_span(self) -> TextEditSpan {
        TextEditSpan {
            start: self.start,
            end: self.end,
            new_text: self.new_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::parsing;

    const FULL_FIXTURE: &str = "Section:\n\n    - item one   \n\n\n\n\n  - item two\n\n";
    const RANGE_FIXTURE: &str =
        "Intro:\n\n    - keep   \n\n\n\n\n  - align me\n\nTail:\n\n\n-  fix me\n- also me\n\n";

    fn parse(source: &str) -> Document {
        parsing::parse_document(source).expect("parse fixture")
    }

    fn apply_spans(source: &str, edits: &[TextEditSpan]) -> String {
        let mut result = source.to_string();
        let mut sorted = edits.to_vec();
        sorted.sort_by_key(|edit| edit.start);
        for edit in sorted.into_iter().rev() {
            result.replace_range(edit.start..edit.end, &edit.new_text);
        }
        result
    }

    #[test]
    fn formats_entire_document() {
        let source = FULL_FIXTURE;
        let document = parse(source);
        let formatted = serialize_to_lex(&document).unwrap();
        assert_ne!(formatted, source);

        let edits = format_document(&document, source);
        assert!(!edits.is_empty());
        let applied = apply_spans(source, &edits);
        assert_eq!(applied, formatted);
    }

    #[test]
    fn range_formatting_filters_edits_outside_selection() {
        let source = RANGE_FIXTURE;
        let document = parse(source);
        let range = LineRange { start: 9, end: 14 };
        let edits = format_range(&document, source, range);
        assert!(!edits.is_empty());

        let locator = SourceLocation::new(source);
        for edit in &edits {
            let start_line = locator.byte_to_position(edit.start).line;
            let end_line = locator.byte_to_position(edit.end).line;
            assert!(start_line >= range.start);
            assert!(end_line <= range.end);
        }

        let applied = apply_spans(source, &edits);
        let start_offset = locator.line_start(range.start).unwrap_or(source.len());
        let end_offset = locator.line_start(range.end).unwrap_or(source.len());

        let prefix_original = &source[..start_offset];
        let suffix_original = &source[end_offset..];

        assert!(applied.starts_with(prefix_original));
        assert!(applied.ends_with(suffix_original));

        let applied_selection = &applied[start_offset..applied.len() - suffix_original.len()];
        let original_selection = &source[start_offset..end_offset];
        assert_ne!(applied_selection, original_selection);
    }

    #[test]
    fn no_edits_when_already_formatted() {
        let source = "Section:\n    - item\n";
        let document = parse(source);
        let edits = format_document(&document, source);
        assert!(edits.is_empty());
    }
}
