//! AST Builder from ParseNode IR
//!
//! This module contains the `AstBuilder`, which walks the `ParseNode` tree
//! produced by the parser and constructs the final AST.

use crate::lex::ast::{AstNode, ContentItem, Document, ListItem, Range};
use crate::lex::building::api as ast_builder;
use crate::lex::building::location::compute_location_from_locations;
use crate::lex::parsing::ir::{NodeType, ParseNode, TokenLocation};

/// A builder that constructs an AST from a `ParseNode` tree.
pub struct AstBuilder<'a> {
    source: &'a str,
}

impl<'a> AstBuilder<'a> {
    /// Creates a new `AstBuilder`.
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    /// Builds a `Document` from a root `ParseNode`.
    pub fn build(&self, root_node: ParseNode) -> Document {
        if root_node.node_type != NodeType::Document {
            // Or handle this more gracefully
            panic!("Expected a Document node at the root");
        }
        let content = self.build_content_items(root_node.children);
        let content_locations: Vec<Range> =
            content.iter().map(|item| item.range().clone()).collect();
        let root_location = compute_location_from_locations(&content_locations);
        Document::with_content(content).with_root_location(root_location)
    }

    /// Builds a vector of `ContentItem`s from a vector of `ParseNode`s.
    fn build_content_items(&self, nodes: Vec<ParseNode>) -> Vec<ContentItem> {
        nodes
            .into_iter()
            .map(|node| self.build_content_item(node))
            .collect()
    }

    /// Builds a single `ContentItem` from a `ParseNode`.
    fn build_content_item(&self, node: ParseNode) -> ContentItem {
        match node.node_type {
            NodeType::Paragraph => self.build_paragraph(node),
            NodeType::Session => self.build_session(node),
            NodeType::List => self.build_list(node),
            NodeType::Definition => self.build_definition(node),
            NodeType::Annotation => self.build_annotation(node),
            NodeType::VerbatimBlock => self.build_verbatim_block(node),
            _ => panic!("Unexpected node type"),
        }
    }

    fn build_paragraph(&self, node: ParseNode) -> ContentItem {
        let token_lines = group_tokens_by_line(node.tokens);
        ast_builder::build_paragraph_from_tokens(token_lines, self.source)
    }

    fn build_session(&self, node: ParseNode) -> ContentItem {
        let title_tokens = node.tokens;
        let content = self.build_content_items(node.children);
        ast_builder::build_session_from_tokens(title_tokens, content, self.source)
    }

    fn build_definition(&self, node: ParseNode) -> ContentItem {
        let subject_tokens = node.tokens;
        let content = self.build_content_items(node.children);
        ast_builder::build_definition_from_tokens(subject_tokens, content, self.source)
    }

    fn build_list(&self, node: ParseNode) -> ContentItem {
        let list_items = node
            .children
            .into_iter()
            .map(|child_node| self.build_list_item(child_node))
            .collect();
        ast_builder::build_list(list_items)
    }

    fn build_list_item(&self, node: ParseNode) -> ListItem {
        let marker_tokens = node.tokens;
        let content = self.build_content_items(node.children);
        ast_builder::build_list_item_from_tokens(marker_tokens, content, self.source)
    }

    fn build_annotation(&self, node: ParseNode) -> ContentItem {
        let header_tokens = node.tokens;
        let content = self.build_content_items(node.children);
        ast_builder::build_annotation_from_tokens(header_tokens, content, self.source)
    }

    fn build_verbatim_block(&self, node: ParseNode) -> ContentItem {
        let mut subject_node = None;
        let mut content_node = None;
        let mut closing_node = None;

        for child in node.children {
            match child.node_type {
                NodeType::VerbatimBlockkSubject => subject_node = Some(child),
                NodeType::VerbatimBlockkContent => content_node = Some(child),
                NodeType::VerbatimBlockkClosing => closing_node = Some(child),
                _ => {}
            }
        }

        let subject_tokens = subject_node.unwrap().tokens;
        let content_token_lines = group_tokens_by_line(content_node.unwrap().tokens);
        let closing_annotation_node = closing_node.unwrap();
        let closing_annotation =
            if let ContentItem::Annotation(ann) = self.build_annotation(closing_annotation_node) {
                ann
            } else {
                panic!("Expected Annotation for verbatim block closing");
            };

        ast_builder::build_verbatim_block_from_tokens(
            subject_tokens,
            content_token_lines,
            closing_annotation,
            self.source,
        )
    }
}

/// Group a flat vector of tokens into lines (split by Newline tokens).
fn group_tokens_by_line(tokens: Vec<TokenLocation>) -> Vec<Vec<TokenLocation>> {
    if tokens.is_empty() {
        return vec![];
    }

    let mut lines: Vec<Vec<TokenLocation>> = vec![];
    let mut current_line: Vec<TokenLocation> = vec![];

    for token_location in tokens {
        if matches!(token_location.0, crate::lex::lexing::Token::BlankLine(_)) {
            lines.push(current_line);
            current_line = vec![];
        } else {
            current_line.push(token_location);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
