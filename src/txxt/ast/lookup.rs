use super::elements::Document;
use super::location::Position;
use super::traits::AstNode;

pub fn find_nodes_at_position(document: &Document, position: Position) -> Vec<&dyn AstNode> {
    if let Some(item) = document.element_at(position) {
        vec![item as &dyn AstNode]
    } else {
        Vec::new()
    }
}

pub fn format_at_position(document: &Document, position: Position) -> String {
    let nodes = find_nodes_at_position(document, position);
    if nodes.is_empty() {
        "No AST nodes at this position".to_string()
    } else {
        nodes
            .iter()
            .map(|node| format!("- {}: {}", node.node_type(), node.display_label()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
