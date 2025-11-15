//! Converts a flat event stream back to a nested IR tree structure.
//!
//! # The High-Level Concept
//!
//! The core challenge is to reconstruct a tree structure from a linear sequence of events.
//! The algorithm uses a stack to keep track of the current nesting level. The stack acts as
//! a memory of "open" containers. When we encounter a `Start` event for a container (like a
//! heading or list), we push it onto the stack, making it the new "current" container. When
//! we see its corresponding `End` event, we pop it off, returning to the parent container.
//!
//! # The Algorithm
//!
//! 1. **Initialization:**
//!    - Create the root `Document` node
//!    - Create an empty stack
//!    - Push the root onto the stack as the current container
//!
//! 2. **Processing `Start` Events:**
//!    - Create a new empty `DocNode` for that element
//!    - Add it as a child to the current parent (top of stack)
//!    - Push it onto the stack as the new current container
//!
//! 3. **Processing Content Events (Inline):**
//!    - Add the content to the current parent (top of stack)
//!    - Do NOT modify the stack (content is a leaf)
//!
//! 4. **Processing `End` Events:**
//!    - Pop the node off the stack
//!    - Validate that the popped node matches the End event
//!
//! 5. **Completion:**
//!    - The stack should contain only the root Document node
//!    - This root contains the complete reconstructed AST

use crate::ir::events::Event;
use crate::ir::nodes::*;

/// Error type for flat-to-nested conversion
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    /// Stack was empty when trying to pop
    UnexpectedEnd(String),
    /// Mismatched start/end events
    MismatchedEvents { expected: String, found: String },
    /// Unexpected inline content in wrong context
    UnexpectedInline(String),
    /// Events remaining after document end
    ExtraEvents,
    /// Stack not empty at end (unclosed containers)
    UnclosedContainers(usize),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnexpectedEnd(msg) => write!(f, "Unexpected end event: {}", msg),
            ConversionError::MismatchedEvents { expected, found } => {
                write!(
                    f,
                    "Mismatched events: expected {}, found {}",
                    expected, found
                )
            }
            ConversionError::UnexpectedInline(msg) => {
                write!(f, "Unexpected inline content: {}", msg)
            }
            ConversionError::ExtraEvents => write!(f, "Extra events after document end"),
            ConversionError::UnclosedContainers(count) => {
                write!(f, "Unclosed containers: {} nodes remain on stack", count)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

/// Represents a node being built on the stack
#[derive(Debug)]
enum StackNode {
    Document(Document),
    Heading {
        level: usize,
        content: Vec<InlineContent>,
        children: Vec<DocNode>,
    },
    Paragraph {
        content: Vec<InlineContent>,
    },
    List {
        items: Vec<ListItem>,
    },
    ListItem {
        content: Vec<InlineContent>,
        children: Vec<DocNode>,
    },
    Definition {
        term: Vec<InlineContent>,
        description: Vec<DocNode>,
        in_term: bool,
    },
    Verbatim {
        language: Option<String>,
        content: String,
    },
    Annotation {
        label: String,
        parameters: Vec<(String, String)>,
        content: Vec<DocNode>,
    },
}

impl StackNode {
    /// Convert to a DocNode (used when popping from stack)
    fn into_doc_node(self) -> DocNode {
        match self {
            StackNode::Document(doc) => DocNode::Document(doc),
            StackNode::Heading {
                level,
                content,
                children,
            } => DocNode::Heading(Heading {
                level,
                content,
                children,
            }),
            StackNode::Paragraph { content } => DocNode::Paragraph(Paragraph { content }),
            StackNode::List { items } => DocNode::List(List { items }),
            StackNode::ListItem { content, children } => {
                DocNode::ListItem(ListItem { content, children })
            }
            StackNode::Definition {
                term, description, ..
            } => DocNode::Definition(Definition { term, description }),
            StackNode::Verbatim { language, content } => {
                DocNode::Verbatim(Verbatim { language, content })
            }
            StackNode::Annotation {
                label,
                parameters,
                content,
            } => DocNode::Annotation(Annotation {
                label,
                parameters,
                content,
            }),
        }
    }

    /// Get the node type name for error messages
    fn type_name(&self) -> &str {
        match self {
            StackNode::Document(_) => "Document",
            StackNode::Heading { .. } => "Heading",
            StackNode::Paragraph { .. } => "Paragraph",
            StackNode::List { .. } => "List",
            StackNode::ListItem { .. } => "ListItem",
            StackNode::Definition { .. } => "Definition",
            StackNode::Verbatim { .. } => "Verbatim",
            StackNode::Annotation { .. } => "Annotation",
        }
    }

    /// Add a child DocNode to this container
    fn add_child(&mut self, child: DocNode) -> Result<(), ConversionError> {
        match self {
            StackNode::Document(doc) => {
                doc.children.push(child);
                Ok(())
            }
            StackNode::Heading { children, .. } => {
                children.push(child);
                Ok(())
            }
            StackNode::ListItem { children, .. } => {
                children.push(child);
                Ok(())
            }
            StackNode::List { items } => {
                if let DocNode::ListItem(item) = child {
                    items.push(item);
                    Ok(())
                } else {
                    Err(ConversionError::MismatchedEvents {
                        expected: "ListItem".to_string(),
                        found: format!("{:?}", child),
                    })
                }
            }
            StackNode::Definition {
                description,
                in_term,
                ..
            } => {
                if *in_term {
                    Err(ConversionError::UnexpectedInline(
                        "Cannot add child to definition term".to_string(),
                    ))
                } else {
                    description.push(child);
                    Ok(())
                }
            }
            StackNode::Annotation { content, .. } => {
                content.push(child);
                Ok(())
            }
            _ => Err(ConversionError::UnexpectedInline(format!(
                "Node {} cannot have children",
                self.type_name()
            ))),
        }
    }

    /// Add inline content to this node
    fn add_inline(&mut self, inline: InlineContent) -> Result<(), ConversionError> {
        match self {
            StackNode::Heading { content, .. } => {
                content.push(inline);
                Ok(())
            }
            StackNode::Paragraph { content } => {
                content.push(inline);
                Ok(())
            }
            StackNode::ListItem { content, .. } => {
                content.push(inline);
                Ok(())
            }
            StackNode::Definition { term, in_term, .. } => {
                if *in_term {
                    term.push(inline);
                    Ok(())
                } else {
                    Err(ConversionError::UnexpectedInline(
                        "Inline content in definition description".to_string(),
                    ))
                }
            }
            StackNode::Verbatim { content, .. } => {
                if let InlineContent::Text(text) = inline {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&text);
                    Ok(())
                } else {
                    Err(ConversionError::UnexpectedInline(
                        "Verbatim can only contain plain text".to_string(),
                    ))
                }
            }
            _ => Err(ConversionError::UnexpectedInline(format!(
                "Cannot add inline content to {}",
                self.type_name()
            ))),
        }
    }
}

/// Converts a flat event stream back to a nested IR tree.
///
/// # Arguments
///
/// * `events` - The flat sequence of events to process
///
/// # Returns
///
/// * `Ok(Document)` - The reconstructed document tree
/// * `Err(ConversionError)` - If the event stream is malformed
///
/// # Example
///
/// ```ignore
/// use lex_babel::ir::events::Event;
/// use lex_babel::mappings::flat_to_nested::events_to_tree;
///
/// let events = vec![
///     Event::StartDocument,
///     Event::StartParagraph,
///     Event::Inline(InlineContent::Text("Hello".to_string())),
///     Event::EndParagraph,
///     Event::EndDocument,
/// ];
///
/// let doc = events_to_tree(&events)?;
/// assert_eq!(doc.children.len(), 1);
/// ```
pub fn events_to_tree(events: &[Event]) -> Result<Document, ConversionError> {
    if events.is_empty() {
        return Ok(Document { children: vec![] });
    }

    let mut stack: Vec<StackNode> = Vec::new();
    let mut event_iter = events.iter().peekable();

    // Expect StartDocument as first event
    match event_iter.next() {
        Some(Event::StartDocument) => {
            stack.push(StackNode::Document(Document { children: vec![] }));
        }
        Some(other) => {
            return Err(ConversionError::MismatchedEvents {
                expected: "StartDocument".to_string(),
                found: format!("{:?}", other),
            });
        }
        None => return Ok(Document { children: vec![] }),
    }

    // Process events
    while let Some(event) = event_iter.next() {
        match event {
            Event::StartDocument => {
                return Err(ConversionError::MismatchedEvents {
                    expected: "content or EndDocument".to_string(),
                    found: "StartDocument".to_string(),
                });
            }

            Event::EndDocument => {
                // Pop the document from stack
                if stack.len() != 1 {
                    return Err(ConversionError::UnclosedContainers(stack.len() - 1));
                }
                let doc_node = stack.pop().unwrap();
                if let StackNode::Document(doc) = doc_node {
                    // Check for extra events
                    if event_iter.peek().is_some() {
                        return Err(ConversionError::ExtraEvents);
                    }
                    return Ok(doc);
                } else {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Document".to_string(),
                        found: doc_node.type_name().to_string(),
                    });
                }
            }

            Event::StartHeading(level) => {
                let node = StackNode::Heading {
                    level: *level,
                    content: vec![],
                    children: vec![],
                };
                stack.push(node);
            }

            Event::EndHeading(level) => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndHeading with empty stack".to_string())
                })?;

                if let StackNode::Heading {
                    level: node_level, ..
                } = &node
                {
                    if node_level != level {
                        return Err(ConversionError::MismatchedEvents {
                            expected: format!("EndHeading({})", node_level),
                            found: format!("EndHeading({})", level),
                        });
                    }
                } else {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Heading".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for heading".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartParagraph => {
                stack.push(StackNode::Paragraph { content: vec![] });
            }

            Event::EndParagraph => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndParagraph with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::Paragraph { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Paragraph".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for paragraph".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartList => {
                stack.push(StackNode::List { items: vec![] });
            }

            Event::EndList => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndList with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::List { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "List".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for list".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartListItem => {
                stack.push(StackNode::ListItem {
                    content: vec![],
                    children: vec![],
                });
            }

            Event::EndListItem => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndListItem with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::ListItem { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "ListItem".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for list item".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartDefinition => {
                stack.push(StackNode::Definition {
                    term: vec![],
                    description: vec![],
                    in_term: false,
                });
            }

            Event::EndDefinition => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndDefinition with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::Definition { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Definition".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for definition".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartDefinitionTerm => {
                if let Some(StackNode::Definition { in_term, .. }) = stack.last_mut() {
                    *in_term = true;
                } else {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Definition on stack".to_string(),
                        found: "StartDefinitionTerm".to_string(),
                    });
                }
            }

            Event::EndDefinitionTerm => {
                if let Some(StackNode::Definition { in_term, .. }) = stack.last_mut() {
                    *in_term = false;
                } else {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Definition on stack".to_string(),
                        found: "EndDefinitionTerm".to_string(),
                    });
                }
            }

            Event::StartDefinitionDescription => {
                // Just a marker, definition is already in description mode after EndDefinitionTerm
            }

            Event::EndDefinitionDescription => {
                // Just a marker, no action needed
            }

            Event::StartVerbatim(language) => {
                stack.push(StackNode::Verbatim {
                    language: language.clone(),
                    content: String::new(),
                });
            }

            Event::EndVerbatim => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndVerbatim with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::Verbatim { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Verbatim".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for verbatim".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::StartAnnotation { label, parameters } => {
                stack.push(StackNode::Annotation {
                    label: label.clone(),
                    parameters: parameters.clone(),
                    content: vec![],
                });
            }

            Event::EndAnnotation => {
                let node = stack.pop().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("EndAnnotation with empty stack".to_string())
                })?;

                if !matches!(node, StackNode::Annotation { .. }) {
                    return Err(ConversionError::MismatchedEvents {
                        expected: "Annotation".to_string(),
                        found: node.type_name().to_string(),
                    });
                }

                let doc_node = node.into_doc_node();
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedEnd("No parent for annotation".to_string())
                })?;
                parent.add_child(doc_node)?;
            }

            Event::Inline(inline) => {
                let parent = stack.last_mut().ok_or_else(|| {
                    ConversionError::UnexpectedInline("Inline content with no parent".to_string())
                })?;
                parent.add_inline(inline.clone())?;
            }
        }
    }

    // If we reach here, document wasn't properly closed
    Err(ConversionError::UnclosedContainers(stack.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_document() {
        let events = vec![Event::StartDocument, Event::EndDocument];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 0);
    }

    #[test]
    fn test_simple_paragraph() {
        let events = vec![
            Event::StartDocument,
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Hello world".to_string())),
            Event::EndParagraph,
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Paragraph(para) => {
                assert_eq!(para.content.len(), 1);
                assert!(matches!(&para.content[0], InlineContent::Text(t) if t == "Hello world"));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_heading_with_content() {
        let events = vec![
            Event::StartDocument,
            Event::StartHeading(1),
            Event::Inline(InlineContent::Text("Title".to_string())),
            Event::EndHeading(1),
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Heading(heading) => {
                assert_eq!(heading.level, 1);
                assert_eq!(heading.content.len(), 1);
                assert!(heading.children.is_empty());
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_nested_heading_with_paragraph() {
        let events = vec![
            Event::StartDocument,
            Event::StartHeading(1),
            Event::Inline(InlineContent::Text("Title".to_string())),
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Content".to_string())),
            Event::EndParagraph,
            Event::EndHeading(1),
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Heading(heading) => {
                assert_eq!(heading.level, 1);
                assert_eq!(heading.children.len(), 1);
                assert!(matches!(&heading.children[0], DocNode::Paragraph(_)));
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_list_with_items() {
        let events = vec![
            Event::StartDocument,
            Event::StartList,
            Event::StartListItem,
            Event::Inline(InlineContent::Text("Item 1".to_string())),
            Event::EndListItem,
            Event::StartListItem,
            Event::Inline(InlineContent::Text("Item 2".to_string())),
            Event::EndListItem,
            Event::EndList,
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::List(list) => {
                assert_eq!(list.items.len(), 2);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_definition() {
        let events = vec![
            Event::StartDocument,
            Event::StartDefinition,
            Event::StartDefinitionTerm,
            Event::Inline(InlineContent::Text("Term".to_string())),
            Event::EndDefinitionTerm,
            Event::StartDefinitionDescription,
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Description".to_string())),
            Event::EndParagraph,
            Event::EndDefinitionDescription,
            Event::EndDefinition,
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Definition(def) => {
                assert_eq!(def.term.len(), 1);
                assert_eq!(def.description.len(), 1);
            }
            _ => panic!("Expected Definition"),
        }
    }

    #[test]
    fn test_verbatim() {
        let events = vec![
            Event::StartDocument,
            Event::StartVerbatim(Some("rust".to_string())),
            Event::Inline(InlineContent::Text("fn main() {}".to_string())),
            Event::EndVerbatim,
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Verbatim(verb) => {
                assert_eq!(verb.language, Some("rust".to_string()));
                assert_eq!(verb.content, "fn main() {}");
            }
            _ => panic!("Expected Verbatim"),
        }
    }

    #[test]
    fn test_annotation() {
        let events = vec![
            Event::StartDocument,
            Event::StartAnnotation {
                label: "note".to_string(),
                parameters: vec![("type".to_string(), "warning".to_string())],
            },
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Warning text".to_string())),
            Event::EndParagraph,
            Event::EndAnnotation,
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Annotation(anno) => {
                assert_eq!(anno.label, "note");
                assert_eq!(anno.parameters.len(), 1);
                assert_eq!(anno.content.len(), 1);
            }
            _ => panic!("Expected Annotation"),
        }
    }

    #[test]
    fn test_complex_nested_document() {
        let events = vec![
            Event::StartDocument,
            Event::StartHeading(1),
            Event::Inline(InlineContent::Text("Chapter 1".to_string())),
            Event::StartHeading(2),
            Event::Inline(InlineContent::Text("Section 1.1".to_string())),
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Some text".to_string())),
            Event::EndParagraph,
            Event::StartList,
            Event::StartListItem,
            Event::Inline(InlineContent::Text("Item".to_string())),
            Event::EndListItem,
            Event::EndList,
            Event::EndHeading(2),
            Event::EndHeading(1),
            Event::EndDocument,
        ];

        let doc = events_to_tree(&events).unwrap();
        assert_eq!(doc.children.len(), 1);

        match &doc.children[0] {
            DocNode::Heading(h1) => {
                assert_eq!(h1.level, 1);
                assert_eq!(h1.children.len(), 1);

                match &h1.children[0] {
                    DocNode::Heading(h2) => {
                        assert_eq!(h2.level, 2);
                        assert_eq!(h2.children.len(), 2); // paragraph and list
                    }
                    _ => panic!("Expected nested Heading"),
                }
            }
            _ => panic!("Expected top Heading"),
        }
    }

    #[test]
    fn test_error_mismatched_end() {
        let events = vec![
            Event::StartDocument,
            Event::StartParagraph,
            Event::EndHeading(1), // Wrong end!
        ];

        let result = events_to_tree(&events);
        assert!(matches!(
            result,
            Err(ConversionError::MismatchedEvents { .. })
        ));
    }

    #[test]
    fn test_error_unclosed_container() {
        let events = vec![
            Event::StartDocument,
            Event::StartParagraph,
            Event::EndDocument, // Missing EndParagraph
        ];

        let result = events_to_tree(&events);
        assert!(matches!(
            result,
            Err(ConversionError::UnclosedContainers(_))
        ));
    }

    #[test]
    fn test_error_extra_events() {
        let events = vec![
            Event::StartDocument,
            Event::EndDocument,
            Event::StartParagraph, // Extra after end!
        ];

        let result = events_to_tree(&events);
        assert!(matches!(result, Err(ConversionError::ExtraEvents)));
    }

    #[test]
    fn test_error_mismatched_heading_level() {
        let events = vec![
            Event::StartDocument,
            Event::StartHeading(1),
            Event::EndHeading(2), // Wrong level!
            Event::EndDocument,
        ];

        let result = events_to_tree(&events);
        assert!(matches!(
            result,
            Err(ConversionError::MismatchedEvents { .. })
        ));
    }

    #[test]
    fn test_round_trip() {
        use crate::ir::to_events::tree_to_events;

        let original_doc = Document {
            children: vec![DocNode::Heading(Heading {
                level: 1,
                content: vec![InlineContent::Text("Title".to_string())],
                children: vec![DocNode::Paragraph(Paragraph {
                    content: vec![InlineContent::Text("Content".to_string())],
                })],
            })],
        };

        // Convert to events
        let events = tree_to_events(&DocNode::Document(original_doc.clone()));

        // Convert back to tree
        let reconstructed = events_to_tree(&events).unwrap();

        // Should match
        assert_eq!(original_doc, reconstructed);
    }

    #[test]
    fn test_round_trip_complex() {
        use crate::ir::to_events::tree_to_events;

        let original_doc = Document {
            children: vec![DocNode::Heading(Heading {
                level: 1,
                content: vec![
                    InlineContent::Text("Title ".to_string()),
                    InlineContent::Bold(vec![InlineContent::Text("bold".to_string())]),
                ],
                children: vec![
                    DocNode::List(List {
                        items: vec![
                            ListItem {
                                content: vec![InlineContent::Text("Item 1".to_string())],
                                children: vec![],
                            },
                            ListItem {
                                content: vec![InlineContent::Text("Item 2".to_string())],
                                children: vec![DocNode::Paragraph(Paragraph {
                                    content: vec![InlineContent::Text("Nested".to_string())],
                                })],
                            },
                        ],
                    }),
                    DocNode::Definition(Definition {
                        term: vec![InlineContent::Text("Term".to_string())],
                        description: vec![DocNode::Paragraph(Paragraph {
                            content: vec![InlineContent::Text("Desc".to_string())],
                        })],
                    }),
                ],
            })],
        };

        let events = tree_to_events(&DocNode::Document(original_doc.clone()));
        let reconstructed = events_to_tree(&events).unwrap();

        assert_eq!(original_doc, reconstructed);
    }
}
