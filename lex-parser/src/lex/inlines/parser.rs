use super::nodes::{InlineContent, InlineNode};

/// Parse inline nodes from a raw string.
pub fn parse_inlines(text: &str) -> InlineContent {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut stack = vec![InlineFrame::new(FrameKind::Root)];
    let mut blocked = BlockedClosings::default();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let prev = if i == 0 { None } else { Some(chars[i - 1]) };
        let next = if i + 1 < chars.len() {
            Some(chars[i + 1])
        } else {
            None
        };

        // Handle escapes first so escaped tokens never trigger parser state.
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
        let top_kind = stack.last().unwrap().kind;

        if let Some(token_kind) = FrameKind::from_char(ch) {
            if token_kind == top_kind && token_kind != FrameKind::Root {
                if blocked.consume(token_kind) {
                    // Literal closing paired to a disallowed nested start.
                } else if is_valid_end(prev, next, token_kind) {
                    let mut frame = stack.pop().unwrap();
                    let had_content = frame.has_content();
                    frame.flush_buffer();
                    if !had_content {
                        // No content -> treat both delimiters as literal.
                        let parent = stack.last_mut().unwrap();
                        parent.push_char(token_kind.token_char().unwrap());
                        parent.push_char(ch);
                    } else {
                        let node = frame.into_node();
                        stack.last_mut().unwrap().push_node(node);
                    }
                    consumed = true;
                } else {
                    // Invalid closing context -> literal character.
                }
            }

            if !consumed
                && !top_kind.is_literal()
                && token_kind != FrameKind::Root
                && is_valid_start(prev, next)
            {
                if stack.iter().any(|frame| frame.kind == token_kind) {
                    blocked.increment(token_kind);
                } else {
                    stack.last_mut().unwrap().flush_buffer();
                    stack.push(InlineFrame::new(token_kind));
                    consumed = true;
                }
            }
        }

        if !consumed {
            stack.last_mut().unwrap().push_char(ch);
        }

        i += 1;
    }

    // Flush any remaining text in the top frame before unwinding.
    if let Some(frame) = stack.last_mut() {
        frame.flush_buffer();
    }

    while stack.len() > 1 {
        let mut frame = stack.pop().unwrap();
        frame.flush_buffer();
        let token_char = frame.kind.token_char().unwrap();
        let parent = stack.last_mut().unwrap();
        parent.push_char(token_char);
        for child in frame.children {
            parent.push_node(child);
        }
    }

    let mut root = stack.pop().unwrap();
    root.flush_buffer();
    root.children
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    Root,
    Strong,
    Emphasis,
    Code,
    Math,
}

impl FrameKind {
    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '*' => Some(FrameKind::Strong),
            '_' => Some(FrameKind::Emphasis),
            '`' => Some(FrameKind::Code),
            '#' => Some(FrameKind::Math),
            _ => None,
        }
    }

    fn token_char(self) -> Option<char> {
        match self {
            FrameKind::Strong => Some('*'),
            FrameKind::Emphasis => Some('_'),
            FrameKind::Code => Some('`'),
            FrameKind::Math => Some('#'),
            FrameKind::Root => None,
        }
    }

    fn is_literal(self) -> bool {
        matches!(self, FrameKind::Code | FrameKind::Math)
    }
}

struct InlineFrame {
    kind: FrameKind,
    buffer: String,
    children: InlineContent,
}

impl InlineFrame {
    fn new(kind: FrameKind) -> Self {
        Self {
            kind,
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

    fn into_node(self) -> InlineNode {
        match self.kind {
            FrameKind::Root => panic!("Cannot convert root frame into inline node"),
            FrameKind::Strong => InlineNode::Strong(self.children),
            FrameKind::Emphasis => InlineNode::Emphasis(self.children),
            FrameKind::Code => InlineNode::Code(flatten_literal(self.children)),
            FrameKind::Math => InlineNode::Math(flatten_literal(self.children)),
        }
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

#[derive(Default)]
struct BlockedClosings {
    strong: usize,
    emphasis: usize,
}

impl BlockedClosings {
    fn increment(&mut self, kind: FrameKind) {
        match kind {
            FrameKind::Strong => self.strong += 1,
            FrameKind::Emphasis => self.emphasis += 1,
            _ => {}
        }
    }

    fn consume(&mut self, kind: FrameKind) -> bool {
        match kind {
            FrameKind::Strong => {
                if self.strong > 0 {
                    self.strong -= 1;
                    true
                } else {
                    false
                }
            }
            FrameKind::Emphasis => {
                if self.emphasis > 0 {
                    self.emphasis -= 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn is_valid_start(prev: Option<char>, next: Option<char>) -> bool {
    !is_word(prev) && is_word(next)
}

fn is_valid_end(prev: Option<char>, next: Option<char>, token: FrameKind) -> bool {
    let inside_valid = match token {
        FrameKind::Code | FrameKind::Math => prev.is_some(),
        _ => matches!(prev, Some(ch) if !ch.is_whitespace()),
    };

    inside_valid && !is_word(next)
}

fn is_word(ch: Option<char>) -> bool {
    ch.map(|c| c.is_alphanumeric()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn escaped_tokens_are_literal() {
        let nodes = parse_inlines("\\*literal\\*");
        assert_eq!(nodes, vec![InlineNode::Plain("*literal*".into())]);
    }
}
