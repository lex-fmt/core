use super::formatting_rules::FormattingRules;
use lex_parser::lex::ast::{
    elements::{
        blank_line_group::BlankLineGroup, paragraph::TextLine, verbatim::VerbatimGroupItemRef,
        VerbatimLine,
    },
    traits::{AstNode, Visitor},
    Annotation, Definition, Document, List, ListItem, Paragraph, Session, Verbatim,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum MarkerType {
    Bullet,
    Numeric,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
    Unknown,
}

struct ListContext {
    index: usize,
    marker_type: MarkerType,
}

impl MarkerType {
    fn from_str(s: &str) -> Self {
        if s == "-" {
            MarkerType::Bullet
        } else if let Some(prefix) = s.strip_suffix('.') {
            if prefix.chars().all(|c| c.is_ascii_digit()) {
                MarkerType::Numeric
            } else if prefix.len() == 1 && prefix.chars().next().unwrap().is_ascii_lowercase() {
                MarkerType::AlphaLower
            } else if prefix.len() == 1 && prefix.chars().next().unwrap().is_ascii_uppercase() {
                MarkerType::AlphaUpper
            } else if is_roman_lower(prefix) {
                MarkerType::RomanLower
            } else if is_roman_upper(prefix) {
                MarkerType::RomanUpper
            } else {
                MarkerType::Unknown
            }
        } else {
            MarkerType::Unknown
        }
    }
}

fn is_roman_lower(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'))
}

fn is_roman_upper(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}

fn to_alpha_lower(n: usize) -> String {
    if (1..=26).contains(&n) {
        char::from_u32((n as u32) + 96).unwrap().to_string()
    } else {
        n.to_string()
    }
}

fn to_alpha_upper(n: usize) -> String {
    if (1..=26).contains(&n) {
        char::from_u32((n as u32) + 64).unwrap().to_string()
    } else {
        n.to_string()
    }
}

fn to_roman_lower(n: usize) -> String {
    // Simple implementation for small numbers
    match n {
        1 => "i".to_string(),
        2 => "ii".to_string(),
        3 => "iii".to_string(),
        4 => "iv".to_string(),
        5 => "v".to_string(),
        _ => n.to_string(), // Fallback
    }
}

fn to_roman_upper(n: usize) -> String {
    match n {
        1 => "I".to_string(),
        2 => "II".to_string(),
        3 => "III".to_string(),
        4 => "IV".to_string(),
        5 => "V".to_string(),
        _ => n.to_string(), // Fallback
    }
}

pub struct LexSerializer {
    rules: FormattingRules,
    output: String,
    indent_level: usize,
    consecutive_newlines: usize,
    list_stack: Vec<ListContext>,
}

impl LexSerializer {
    pub fn new(rules: FormattingRules) -> Self {
        Self {
            rules,
            output: String::new(),
            indent_level: 0,
            consecutive_newlines: 2, // Start as if we have blank lines
            list_stack: Vec::new(),
        }
    }

    pub fn serialize(mut self, doc: &Document) -> Result<String, String> {
        doc.root.accept(&mut self);
        Ok(self.output)
    }

    fn indent(&self) -> String {
        self.rules.indent_string.repeat(self.indent_level)
    }

    fn write_line(&mut self, text: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(text);
        self.output.push('\n');
        self.consecutive_newlines = 1;
    }

    fn ensure_blank_lines(&mut self, count: usize) {
        let target_newlines = count + 1;
        while self.consecutive_newlines < target_newlines {
            self.output.push('\n');
            self.consecutive_newlines += 1;
        }
    }
}

impl Visitor for LexSerializer {
    fn visit_session(&mut self, session: &Session) {
        let title = session.title.as_string();
        if !title.is_empty() {
            self.ensure_blank_lines(self.rules.session_blank_lines_before);
            self.write_line(title);
            self.ensure_blank_lines(self.rules.session_blank_lines_after);
            self.indent_level += 1;
        }
    }

    fn leave_session(&mut self, session: &Session) {
        if !session.title.as_string().is_empty() {
            self.indent_level -= 1;
        }
    }

    fn visit_paragraph(&mut self, _paragraph: &Paragraph) {
        // Paragraphs are handled by visiting TextLines
    }

    fn visit_text_line(&mut self, text_line: &TextLine) {
        self.write_line(text_line.text());
    }

    fn visit_blank_line_group(&mut self, group: &BlankLineGroup) {
        let count = if self.rules.max_blank_lines > 0 {
            std::cmp::min(group.count, self.rules.max_blank_lines)
        } else {
            group.count
        };
        self.ensure_blank_lines(count);
    }

    fn visit_list(&mut self, list: &List) {
        let marker_type = if let Some(first) = list.items.iter().next() {
            if let Some(item) = first.as_list_item() {
                MarkerType::from_str(item.marker.as_string())
            } else {
                MarkerType::Bullet
            }
        } else {
            MarkerType::Bullet
        };

        self.list_stack.push(ListContext {
            index: 1,
            marker_type,
        });
    }

    fn leave_list(&mut self, _list: &List) {
        self.list_stack.pop();
    }

    fn visit_list_item(&mut self, list_item: &ListItem) {
        let context = self
            .list_stack
            .last_mut()
            .expect("List stack empty in list item");

        let marker = if self.rules.normalize_list_markers {
            match context.marker_type {
                MarkerType::Bullet => self.rules.unordered_list_marker.to_string(),
                MarkerType::Numeric => format!("{}.", context.index),
                MarkerType::AlphaLower => format!("{}.", to_alpha_lower(context.index)),
                MarkerType::AlphaUpper => format!("{}.", to_alpha_upper(context.index)),
                MarkerType::RomanLower => format!("{}.", to_roman_lower(context.index)),
                MarkerType::RomanUpper => format!("{}.", to_roman_upper(context.index)),
                MarkerType::Unknown => list_item.marker.as_string().to_string(),
            }
        } else {
            list_item.marker.as_string().to_string()
        };

        context.index += 1;

        // Use the first text content as the item line
        let text = if !list_item.text.is_empty() {
            list_item.text[0].as_string()
        } else {
            ""
        };

        let line = if text.is_empty() {
            marker
        } else {
            format!("{} {}", marker, text)
        };

        self.write_line(&line);
        self.indent_level += 1;
    }

    fn leave_list_item(&mut self, _list_item: &ListItem) {
        self.indent_level -= 1;
    }

    fn visit_definition(&mut self, definition: &Definition) {
        let subject = definition.subject.as_string();
        self.write_line(&format!("{}:", subject));
        self.indent_level += 1;
    }

    fn leave_definition(&mut self, _definition: &Definition) {
        self.indent_level -= 1;
    }

    fn visit_annotation(&mut self, annotation: &Annotation) {
        let label = &annotation.data.label.value;
        let params = &annotation.data.parameters;

        let mut header = format!(":: {}", label);
        if !params.is_empty() {
            for param in params {
                header.push(' ');
                header.push_str(&param.key);
                header.push('=');
                header.push_str(&param.value);
            }
        }

        // Only add closing :: for short-form annotations (no children)
        if annotation.children.is_empty() {
            header.push_str(" ::");
        }

        self.write_line(&header);

        if !annotation.children.is_empty() {
            self.indent_level += 1;
        }
    }

    fn leave_annotation(&mut self, annotation: &Annotation) {
        if !annotation.children.is_empty() {
            self.indent_level -= 1;
            self.write_line("::");
        }
    }

    fn visit_verbatim_block(&mut self, _verbatim: &Verbatim) {
        // Handled in groups
    }

    fn visit_verbatim_group(&mut self, group: &VerbatimGroupItemRef) {
        let subject = group.subject.as_string();
        self.write_line(&format!("{}:", subject));
        self.indent_level += 1;
    }

    fn leave_verbatim_group(&mut self, _group: &VerbatimGroupItemRef) {
        self.indent_level -= 1;
    }

    fn visit_verbatim_line(&mut self, verbatim_line: &VerbatimLine) {
        self.write_line(verbatim_line.content.as_string());
    }

    fn leave_verbatim_block(&mut self, verbatim: &Verbatim) {
        let label = &verbatim.closing_data.label.value;
        let mut footer = format!(":: {}", label);
        if !verbatim.closing_data.parameters.is_empty() {
            for param in &verbatim.closing_data.parameters {
                footer.push(' ');
                footer.push_str(&param.key);
                footer.push('=');
                footer.push_str(&param.value);
            }
        }
        self.write_line(&footer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::ast::{ContentItem, Paragraph};

    #[test]
    fn test_serialize_simple_paragraph() {
        let rules = FormattingRules::default();
        let serializer = LexSerializer::new(rules);
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        let result = serializer.serialize(&doc).unwrap();
        assert_eq!(result, "Hello world\n");
    }

    #[test]
    fn test_serialize_session() {
        let rules = FormattingRules::default();
        let serializer = LexSerializer::new(rules);
        let mut session = Session::with_title("Intro".to_string());
        session
            .children
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            )));
        let doc = Document::with_content(vec![ContentItem::Session(session)]);

        let result = serializer.serialize(&doc).unwrap();
        // Expect:
        //
        // Intro
        //
        //     Content
        //
        // Note: ensure_blank_line adds one blank line if not present.
        // Root session (doc) has empty title, so no output for it.
        // Child session "Intro" has title.
        // ensure_blank_line() -> adds \n (because last_was_blank=true initially? No, last_was_blank=true initially).
        // Wait, if last_was_blank=true, ensure_blank_line does nothing.
        // So "Intro" is written.
        // Then ensure_blank_line() -> adds \n.
        // Then indent.
        // Then "Content" indented.

        let expected = "Intro\n\n    Content\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_serialize_list_normalization() {
        use lex_parser::lex::ast::{List, ListItem};

        let rules = FormattingRules::default();
        let serializer = LexSerializer::new(rules);

        // Create list with mixed markers: "1.", "3.", "5."
        let item1 = ListItem::new("1.".to_string(), "One".to_string());
        let item2 = ListItem::new("3.".to_string(), "Two".to_string());
        let item3 = ListItem::new("5.".to_string(), "Three".to_string());

        let list = List::new(vec![item1, item2, item3]);
        let doc = Document::with_content(vec![ContentItem::List(list)]);

        let result = serializer.serialize(&doc).unwrap();

        // Expect sequential numbering: 1. 2. 3.
        let expected = "1. One\n2. Two\n3. Three\n";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_serialize_annotation_short_form() {
        use lex_parser::lex::ast::{elements::label::Label, Annotation, Data};

        let rules = FormattingRules::default();
        let serializer = LexSerializer::new(rules);

        let annotation =
            Annotation::from_data(Data::new(Label::new("note".into()), Vec::new()), Vec::new());
        let doc = Document::with_content(vec![ContentItem::Annotation(annotation)]);

        let result = serializer.serialize(&doc).unwrap();
        assert_eq!(result, ":: note ::\n");
    }

    #[test]
    fn test_serialize_annotation_block_form() {
        use lex_parser::lex::ast::{
            elements::label::Label, elements::typed_content::ContentElement, Annotation, Data,
        };

        let rules = FormattingRules::default();
        let serializer = LexSerializer::new(rules);

        let paragraph = Paragraph::from_line("This is an important note.".to_string());
        let annotation = Annotation::from_data(
            Data::new(Label::new("note".into()), Vec::new()),
            vec![ContentElement::try_from(ContentItem::Paragraph(paragraph)).unwrap()],
        );
        let doc = Document::with_content(vec![ContentItem::Annotation(annotation)]);

        let result = serializer.serialize(&doc).unwrap();
        let expected = ":: note\n    This is an important note.\n::\n";
        assert_eq!(result, expected);
    }
}
