//! Converts a nested IR `DocNode` tree to a flat event stream.

use super::events::Event;
use super::nodes::*;

/// Converts a `DocNode` tree to a flat vector of `Event`s.
pub fn tree_to_events(root_node: &DocNode) -> Vec<Event> {
    let mut events = Vec::new();
    walk_node(root_node, &mut events);
    events
}

fn walk_node(node: &DocNode, events: &mut Vec<Event>) {
    match node {
        DocNode::Document(doc) => {
            events.push(Event::StartDocument);
            for child in &doc.children {
                walk_node(child, events);
            }
            events.push(Event::EndDocument);
        }
        DocNode::Heading(heading) => {
            events.push(Event::StartHeading(heading.level));
            for inline in &heading.content {
                events.push(Event::Inline(inline.clone()));
            }
            for child in &heading.children {
                walk_node(child, events);
            }
            events.push(Event::EndHeading(heading.level));
        }
        DocNode::Paragraph(para) => {
            events.push(Event::StartParagraph);
            for inline in &para.content {
                events.push(Event::Inline(inline.clone()));
            }
            events.push(Event::EndParagraph);
        }
        DocNode::List(list) => {
            events.push(Event::StartList);
            for item in &list.items {
                walk_list_item(item, events);
            }
            events.push(Event::EndList);
        }
        DocNode::ListItem(_) => {
            // ListItems are handled by the List case
        }
        DocNode::Definition(def) => {
            events.push(Event::StartDefinition);
            events.push(Event::StartDefinitionTerm);
            for inline in &def.term {
                events.push(Event::Inline(inline.clone()));
            }
            events.push(Event::EndDefinitionTerm);
            events.push(Event::StartDefinitionDescription);
            for child in &def.description {
                walk_node(child, events);
            }
            events.push(Event::EndDefinitionDescription);
            events.push(Event::EndDefinition);
        }
        DocNode::Verbatim(verb) => {
            events.push(Event::StartVerbatim(verb.language.clone()));
            // Verbatim content is handled as a single inline event in this model
            events.push(Event::Inline(InlineContent::Text(verb.content.clone())));
            events.push(Event::EndVerbatim);
        }
        DocNode::Annotation(anno) => {
            events.push(Event::StartAnnotation {
                label: anno.label.clone(),
                parameters: anno.parameters.clone(),
            });
            for child in &anno.content {
                walk_node(child, events);
            }
            events.push(Event::EndAnnotation);
        }
        DocNode::Inline(inline) => {
            events.push(Event::Inline(inline.clone()));
        }
    }
}

fn walk_list_item(item: &ListItem, events: &mut Vec<Event>) {
    events.push(Event::StartListItem);
    for inline in &item.content {
        events.push(Event::Inline(inline.clone()));
    }
    for child in &item.children {
        walk_node(child, events);
    }
    events.push(Event::EndListItem);
}
