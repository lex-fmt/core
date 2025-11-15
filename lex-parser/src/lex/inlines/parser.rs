use super::nodes::{InlineContent, InlineNode};
use once_cell::sync::Lazy;
use std::collections::HashMap;

static DEFAULT_INLINE_PARSER: Lazy<InlineParser> = Lazy::new(InlineParser::new);

/// Parse inline nodes from a raw string using the default inline parser configuration.
pub fn parse_inlines(text: &str) -> InlineContent {
    DEFAULT_INLINE_PARSER.parse(text)
}

/// Parse inline nodes using a custom parser configuration.
pub fn parse_inlines_with_parser(text: &str, parser: &InlineParser) -> InlineContent {
    parser.parse(text)
}

/// Optional transformation applied to a parsed inline node.
pub type InlinePostProcessor = fn(InlineNode) -> InlineNode;

#[derive(Clone)]
pub struct InlineSpec {
    pub kind: InlineKind,
    pub start_token: char,
    pub end_token: char,
    pub literal: bool,
    pub post_process: Option<InlinePostProcessor>,
}

impl InlineSpec {
    fn apply_post_process(&self, node: InlineNode) -> InlineNode {
        if let Some(callback) = self.post_process {
            callback(node)
        } else {
            node
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlineKind {
    Strong,
    Emphasis,
    Code,
    Math,
}

#[derive(Clone)]
pub struct InlineParser {
    specs: Vec<InlineSpec>,
    token_map: HashMap<char, usize>,
}

impl InlineParser {
    pub fn new() -> Self {
        Self::from_specs(default_specs())
    }

    /// Attach a post-processing callback to a specific inline kind.
    pub fn with_post_processor(mut self, kind: InlineKind, processor: InlinePostProcessor) -> Self {
        if let Some(spec) = self.specs.iter_mut().find(|spec| spec.kind == kind) {
            spec.post_process = Some(processor);
        }
        self
    }

    pub fn parse(&self, text: &str) -> InlineContent {
        parse_with(self, text)
    }

    fn from_specs(specs: Vec<InlineSpec>) -> Self {
        let mut token_map = HashMap::new();
        for (index, spec) in specs.iter().enumerate() {
            token_map.insert(spec.start_token, index);
        }
        Self { specs, token_map }
    }

    fn spec(&self, index: usize) -> &InlineSpec {
        &self.specs[index]
    }

    fn spec_index_for_start(&self, ch: char) -> Option<usize> {
        self.token_map.get(&ch).copied()
    }

    fn spec_count(&self) -> usize {
        self.specs.len()
    }
}

impl Default for InlineParser {
    fn default() -> Self {
        InlineParser::new()
    }
}

fn default_specs() -> Vec<InlineSpec> {
    vec![
        InlineSpec {
            kind: InlineKind::Strong,
            start_token: '*',
            end_token: '*',
            literal: false,
            post_process: None,
        },
        InlineSpec {
            kind: InlineKind::Emphasis,
            start_token: '_',
            end_token: '_',
            literal: false,
            post_process: None,
        },
        InlineSpec {
            kind: InlineKind::Code,
            start_token: '`',
            end_token: '`',
            literal: true,
            post_process: None,
        },
        InlineSpec {
            kind: InlineKind::Math,
            start_token: '#',
            end_token: '#',
            literal: true,
            post_process: None,
        },
    ]
}

fn parse_with(parser: &InlineParser, text: &str) -> InlineContent {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut stack = vec![InlineFrame::root()];
    let mut blocked = BlockedClosings::new(parser.spec_count());

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let prev = if i == 0 { None } else { Some(chars[i - 1]) };
        let next = if i + 1 < chars.len() {
            Some(chars[i + 1])
        } else {
            None
        };

        if ch == '\\' {
            if let Some(next_char) = next {
                stack.last_mut().unwrap().push_char(next_char);
                i += 2;
                continue;
            } else {
                stack.last_mut().unwrap().push_char('\\');
                break;
            }
        }

        let mut consumed = false;
        if let Some(spec_index) = stack.last().unwrap().spec_index {
            let spec = parser.spec(spec_index);
            if ch == spec.end_token {
                if blocked.consume(spec_index) {
                    // Literal closing paired to a disallowed nested start.
                } else if is_valid_end(prev, next, spec) {
                    let mut frame = stack.pop().unwrap();
                    frame.flush_buffer();
                    let had_content = frame.has_content();
                    if !had_content {
                        let parent = stack.last_mut().unwrap();
                        parent.push_char(spec.start_token);
                        parent.push_char(spec.end_token);
                    } else {
                        let node = frame.into_node(spec);
                        let node = spec.apply_post_process(node);
                        stack.last_mut().unwrap().push_node(node);
                    }
                    consumed = true;
                }
            }
        }

        if !consumed && !stack.last().unwrap().is_literal(parser) {
            if let Some(spec_index) = parser.spec_index_for_start(ch) {
                let spec = parser.spec(spec_index);
                if is_valid_start(prev, next, spec) {
                    if stack
                        .iter()
                        .any(|frame| frame.spec_index == Some(spec_index))
                    {
                        blocked.increment(spec_index);
                    } else {
                        stack.last_mut().unwrap().flush_buffer();
                        stack.push(InlineFrame::new(spec_index));
                        consumed = true;
                    }
                }
            }
        }

        if !consumed {
            stack.last_mut().unwrap().push_char(ch);
        }

        i += 1;
    }

    if let Some(frame) = stack.last_mut() {
        frame.flush_buffer();
    }

    while stack.len() > 1 {
        let mut frame = stack.pop().unwrap();
        frame.flush_buffer();
        let spec_index = frame
            .spec_index
            .expect("non-root stack frame must have a spec");
        let spec = parser.spec(spec_index);
        let parent = stack.last_mut().unwrap();
        parent.push_char(spec.start_token);
        for child in frame.children {
            parent.push_node(child);
        }
    }

    let mut root = stack.pop().unwrap();
    root.flush_buffer();
    root.children
}

struct InlineFrame {
    spec_index: Option<usize>,
    buffer: String,
    children: InlineContent,
}

impl InlineFrame {
    fn root() -> Self {
        Self {
            spec_index: None,
            buffer: String::new(),
            children: Vec::new(),
        }
    }

    fn new(spec_index: usize) -> Self {
        Self {
            spec_index: Some(spec_index),
            buffer: String::new(),
            children: Vec::new(),
        }
    }

    fn has_content(&self) -> bool {
        !self.buffer.is_empty() || !self.children.is_empty()
    }

    fn push_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.buffer);
        if let Some(InlineNode::Plain(existing)) = self.children.last_mut() {
            existing.push_str(&text);
        } else {
            self.children.push(InlineNode::Plain(text));
        }
    }

    fn push_node(&mut self, node: InlineNode) {
        self.flush_buffer();
        match node {
            InlineNode::Plain(text) => {
                if text.is_empty() {
                    return;
                }
                if let Some(InlineNode::Plain(existing)) = self.children.last_mut() {
                    existing.push_str(&text);
                } else {
                    self.children.push(InlineNode::Plain(text));
                }
            }
            other => self.children.push(other),
        }
    }

    fn into_node(self, spec: &InlineSpec) -> InlineNode {
        match spec.kind {
            InlineKind::Strong => InlineNode::Strong(self.children),
            InlineKind::Emphasis => InlineNode::Emphasis(self.children),
            InlineKind::Code => InlineNode::Code(flatten_literal(self.children)),
            InlineKind::Math => InlineNode::Math(flatten_literal(self.children)),
        }
    }

    fn is_literal(&self, parser: &InlineParser) -> bool {
        self.spec_index
            .map(|index| parser.spec(index).literal)
            .unwrap_or(false)
    }
}

fn flatten_literal(children: InlineContent) -> String {
    let mut text = String::new();
    for node in children {
        match node {
            InlineNode::Plain(segment) => text.push_str(&segment),
            _ => fatal_literal_content(),
        }
    }
    text
}

fn fatal_literal_content() -> ! {
    panic!("Literal inline nodes must not contain nested nodes");
}

struct BlockedClosings {
    counts: Vec<usize>,
}

impl BlockedClosings {
    fn new(spec_len: usize) -> Self {
        Self {
            counts: vec![0; spec_len],
        }
    }

    fn increment(&mut self, spec_index: usize) {
        if let Some(slot) = self.counts.get_mut(spec_index) {
            *slot += 1;
        }
    }

    fn consume(&mut self, spec_index: usize) -> bool {
        if let Some(slot) = self.counts.get_mut(spec_index) {
            if *slot > 0 {
                *slot -= 1;
                return true;
            }
        }
        false
    }
}

fn is_valid_start(prev: Option<char>, next: Option<char>, _spec: &InlineSpec) -> bool {
    !is_word(prev) && is_word(next)
}

fn is_valid_end(prev: Option<char>, next: Option<char>, spec: &InlineSpec) -> bool {
    let inside_valid = if spec.literal {
        prev.is_some()
    } else {
        matches!(prev, Some(ch) if !ch.is_whitespace())
    };

    inside_valid && !is_word(next)
}

fn is_word(ch: Option<char>) -> bool {
    ch.map(|c| c.is_alphanumeric()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::inlines::InlineNode;

    #[test]
    fn parses_plain_text() {
        let nodes = parse_inlines("hello world");
        assert_eq!(nodes, vec![InlineNode::Plain("hello world".into())]);
    }

    #[test]
    fn parses_strong_and_emphasis() {
        let nodes = parse_inlines("*strong _inner_* text");
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            InlineNode::Strong(children) => {
                assert_eq!(children.len(), 2);
                assert_eq!(children[0], InlineNode::Plain("strong ".into()));
                match &children[1] {
                    InlineNode::Emphasis(inner) => {
                        assert_eq!(inner, &vec![InlineNode::Plain("inner".into())]);
                    }
                    other => panic!("Unexpected child: {:?}", other),
                }
            }
            other => panic!("Unexpected node: {:?}", other),
        }
        assert_eq!(nodes[1], InlineNode::Plain(" text".into()));
    }

    #[test]
    fn nested_emphasis_inside_strong() {
        let nodes = parse_inlines("*strong and _emphasis_* text");
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            InlineNode::Strong(children) => {
                assert_eq!(children.len(), 2);
                assert_eq!(children[0], InlineNode::Plain("strong and ".into()));
                match &children[1] {
                    InlineNode::Emphasis(inner) => {
                        assert_eq!(inner, &vec![InlineNode::Plain("emphasis".into())]);
                    }
                    other => panic!("Unexpected child: {:?}", other),
                }
            }
            _ => panic!("Expected strong node"),
        }
    }

    #[test]
    fn code_is_literal() {
        let nodes = parse_inlines("`a * literal _` text");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], InlineNode::Code("a * literal _".into()));
        assert_eq!(nodes[1], InlineNode::Plain(" text".into()));
    }

    #[test]
    fn math_is_literal() {
        let nodes = parse_inlines("#x + y#");
        assert_eq!(nodes, vec![InlineNode::Math("x + y".into())]);
    }

    #[test]
    fn unmatched_start_is_literal() {
        let nodes = parse_inlines("prefix *text");
        assert_eq!(nodes, vec![InlineNode::Plain("prefix *text".into())]);
    }

    #[test]
    fn unmatched_nested_preserves_children() {
        let nodes = parse_inlines("*a _b_ c");
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0], InlineNode::Plain("*a ".into()));
        match &nodes[1] {
            InlineNode::Emphasis(children) => {
                assert_eq!(children, &vec![InlineNode::Plain("b".into())]);
            }
            other => panic!("Unexpected node: {:?}", other),
        }
        assert_eq!(nodes[2], InlineNode::Plain(" c".into()));
    }

    #[test]
    fn same_type_nesting_skips_inner_pair() {
        let nodes = parse_inlines("*outer *inner* text*");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            InlineNode::Strong(children) => {
                assert_eq!(
                    children,
                    &vec![InlineNode::Plain("outer *inner* text".into())]
                );
            }
            other => panic!("Unexpected node: {:?}", other),
        }
    }

    fn annotate_strong(node: InlineNode) -> InlineNode {
        match node {
            InlineNode::Strong(mut children) => {
                let mut annotated = vec![InlineNode::Plain("[strong]".into())];
                annotated.append(&mut children);
                InlineNode::Strong(annotated)
            }
            other => other,
        }
    }

    #[test]
    fn post_process_callback_transforms_node() {
        let parser = InlineParser::new().with_post_processor(InlineKind::Strong, annotate_strong);
        let nodes = parser.parse("*bold*");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            InlineNode::Strong(children) => {
                assert_eq!(children[0], InlineNode::Plain("[strong]".into()));
                assert_eq!(children[1], InlineNode::Plain("bold".into()));
            }
            other => panic!("Unexpected inline node: {:?}", other),
        }
    }

    #[test]
    fn escaped_tokens_are_literal() {
        let nodes = parse_inlines("\\*literal\\*");
        assert_eq!(nodes, vec![InlineNode::Plain("*literal*".into())]);
    }
}
