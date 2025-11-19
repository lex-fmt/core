//! Converts a nested IR tree structure into a flat event stream.
//!
//! # The High-Level Concept
//!
//! Traversing the nested document structure in pre-order lets us emit a
//! sequence of start/content/end events that can later be reassembled into
//! the original tree. Each container node produces its own start/end markers
//! and then recurses into children so the flat stream preserves the original
//! nesting.
//!
//! # The Algorithm
//!
//! 1. **Initialization:**
//!    - Create an empty event vector
//!    - Begin walking from the root `DocNode`
//!
//! 2. **Entering Containers:**
//!    - Emit the corresponding `Start*` event
//!    - Emit inline content, if any
//!    - Recurse into child nodes
//!
//! 3. **Handling Inline Nodes:**
//!    - Inline-only nodes become a single `Inline` event in place
//!
//! 4. **Exiting Containers:**
//!    - Emit the matching `End*` event once children are processed
//!
//! 5. **Completion:**
//!    - Return the accumulated event stream
//!
//! This mirrors the reverse process performed in `flat_to_nested`, ensuring
//! round-trippable conversions between the nested IR and flat event stream.

use crate::ir::events::Event;
use crate::ir::nodes::{
    Annotation, Definition, DocNode, Document, Heading, InlineContent, List, ListItem, Paragraph,
    Verbatim,
};

/// Converts a `DocNode` tree to a flat vector of `Event`s.
pub fn tree_to_events(root_node: &DocNode) -> Vec<Event> {
    let mut events = Vec::new();
    walk_node(root_node, &mut events);
    events
}

fn walk_node(node: &DocNode, events: &mut Vec<Event>) {
    match node {
        DocNode::Document(Document { children }) => {
            events.push(Event::StartDocument);
            for child in children {
                walk_node(child, events);
            }
            events.push(Event::EndDocument);
        }
        DocNode::Heading(Heading {
            level,
            content,
            children,
        }) => {
            events.push(Event::StartHeading(*level));
            emit_inlines(content, events);
            for child in children {
                walk_node(child, events);
            }
            events.push(Event::EndHeading(*level));
        }
        DocNode::Paragraph(Paragraph { content }) => {
            events.push(Event::StartParagraph);
            emit_inlines(content, events);
            events.push(Event::EndParagraph);
        }
        DocNode::List(List { items, ordered }) => {
            events.push(Event::StartList { ordered: *ordered });
            for item in items {
                walk_list_item(item, events);
            }
            events.push(Event::EndList);
        }
        DocNode::ListItem(_) => {
            // List items are emitted by the surrounding list handler.
            if cfg!(debug_assertions) {
                unreachable!("ListItem should only be emitted by List");
            }
        }
        DocNode::Definition(Definition { term, description }) => {
            events.push(Event::StartDefinition);
            events.push(Event::StartDefinitionTerm);
            emit_inlines(term, events);
            events.push(Event::EndDefinitionTerm);
            events.push(Event::StartDefinitionDescription);
            for child in description {
                walk_node(child, events);
            }
            events.push(Event::EndDefinitionDescription);
            events.push(Event::EndDefinition);
        }
        DocNode::Verbatim(Verbatim { language, content }) => {
            events.push(Event::StartVerbatim(language.clone()));
            events.push(Event::Inline(InlineContent::Text(content.clone())));
            events.push(Event::EndVerbatim);
        }
        DocNode::Annotation(Annotation {
            label,
            parameters,
            content,
        }) => {
            events.push(Event::StartAnnotation {
                label: label.clone(),
                parameters: parameters.clone(),
            });
            for child in content {
                walk_node(child, events);
            }
            events.push(Event::EndAnnotation);
        }
        DocNode::Inline(inline) => events.push(Event::Inline(inline.clone())),
    }
}

fn walk_list_item(item: &ListItem, events: &mut Vec<Event>) {
    events.push(Event::StartListItem);
    emit_inlines(&item.content, events);
    for child in &item.children {
        walk_node(child, events);
    }
    events.push(Event::EndListItem);
}

fn emit_inlines(inlines: &[InlineContent], events: &mut Vec<Event>) {
    for inline in inlines {
        events.push(Event::Inline(inline.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mappings::flat_to_nested::events_to_tree;

    fn sample_tree() -> DocNode {
        DocNode::Document(Document {
            children: vec![
                DocNode::Heading(Heading {
                    level: 2,
                    content: vec![InlineContent::Text("Intro".to_string())],
                    children: vec![DocNode::Paragraph(Paragraph {
                        content: vec![InlineContent::Text("Welcome".to_string())],
                    })],
                }),
                DocNode::List(List {
                    items: vec![ListItem {
                        content: vec![InlineContent::Text("Item".to_string())],
                        children: vec![DocNode::Verbatim(Verbatim {
                            language: Some("rust".to_string()),
                            content: "fn main() {}".to_string(),
                        })],
                    }],
                    ordered: false,
                }),
                DocNode::Definition(Definition {
                    term: vec![InlineContent::Text("Term".to_string())],
                    description: vec![DocNode::Paragraph(Paragraph {
                        content: vec![InlineContent::Text("Definition".to_string())],
                    })],
                }),
                DocNode::Annotation(Annotation {
                    label: "note".to_string(),
                    parameters: vec![("key".to_string(), "value".to_string())],
                    content: vec![DocNode::Paragraph(Paragraph {
                        content: vec![InlineContent::Text("Body".to_string())],
                    })],
                }),
            ],
        })
    }

    #[test]
    fn flattens_nested_document() {
        let events = tree_to_events(&sample_tree());

        let expected = vec![
            Event::StartDocument,
            Event::StartHeading(2),
            Event::Inline(InlineContent::Text("Intro".to_string())),
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Welcome".to_string())),
            Event::EndParagraph,
            Event::EndHeading(2),
            Event::StartList { ordered: false },
            Event::StartListItem,
            Event::Inline(InlineContent::Text("Item".to_string())),
            Event::StartVerbatim(Some("rust".to_string())),
            Event::Inline(InlineContent::Text("fn main() {}".to_string())),
            Event::EndVerbatim,
            Event::EndListItem,
            Event::EndList,
            Event::StartDefinition,
            Event::StartDefinitionTerm,
            Event::Inline(InlineContent::Text("Term".to_string())),
            Event::EndDefinitionTerm,
            Event::StartDefinitionDescription,
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Definition".to_string())),
            Event::EndParagraph,
            Event::EndDefinitionDescription,
            Event::EndDefinition,
            Event::StartAnnotation {
                label: "note".to_string(),
                parameters: vec![("key".to_string(), "value".to_string())],
            },
            Event::StartParagraph,
            Event::Inline(InlineContent::Text("Body".to_string())),
            Event::EndParagraph,
            Event::EndAnnotation,
            Event::EndDocument,
        ];

        assert_eq!(events, expected);
    }

    #[test]
    fn round_trips_with_flat_to_nested() {
        let original = sample_tree();
        let events = tree_to_events(&original);
        let rebuilt = events_to_tree(&events).expect("failed to rebuild");

        assert_eq!(DocNode::Document(rebuilt), original);
    }
}
