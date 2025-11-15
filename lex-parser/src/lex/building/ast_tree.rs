//! AST Builder from ParseNode IR
//!
//! This module contains the `AstTreeBuilder`, which walks the `ParseNode` tree
//! produced by the parser and constructs the final AST.

use crate::lex::ast::elements::typed_content::{ContentElement, SessionContent};
use crate::lex::ast::error::{format_source_context, ParserError, ParserResult};
use crate::lex::ast::range::SourceLocation;
use crate::lex::ast::{AstNode, ContentItem, Document, ListItem, Range};
use crate::lex::building::api as ast_api;
use crate::lex::building::location::compute_location_from_locations;
use crate::lex::parsing::ir::{NodeType, ParseNode, ParseNodePayload, TokenLocation};

/// A builder that constructs an AST from a `ParseNode` tree.
pub struct AstTreeBuilder<'a> {
    source: &'a str,
    source_location: SourceLocation,
}

impl<'a> AstTreeBuilder<'a> {
    /// Creates a new `AstTreeBuilder`.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            source_location: SourceLocation::new(source),
        }
    }

    /// Builds a `Document` from a root `ParseNode`.
    pub fn build(&self, root_node: ParseNode) -> ParserResult<Document> {
        if root_node.node_type != NodeType::Document {
            // Or handle this more gracefully
            panic!("Expected a Document node at the root");
        }
        let content = self.build_content_items(root_node.children)?;
        let content_locations: Vec<Range> =
            content.iter().map(|item| item.range().clone()).collect();
        let root_location = compute_location_from_locations(&content_locations);
        Ok(Document::with_content(content).with_root_location(root_location))
    }

    /// Builds a vector of `ContentItem`s from a vector of `ParseNode`s.
    fn build_content_items(&self, nodes: Vec<ParseNode>) -> ParserResult<Vec<ContentItem>> {
        nodes
            .into_iter()
            .map(|node| self.build_content_item(node))
            .collect()
    }

    /// Builds a single `ContentItem` from a `ParseNode`.
    fn build_content_item(&self, node: ParseNode) -> ParserResult<ContentItem> {
        match node.node_type {
            NodeType::Paragraph => Ok(self.build_paragraph(node)),
            NodeType::Session => self.build_session(node),
            NodeType::List => self.build_list(node),
            NodeType::Definition => self.build_definition(node),
            NodeType::Annotation => self.build_annotation(node),
            NodeType::VerbatimBlock => Ok(self.build_verbatim_block(node)),
            NodeType::BlankLineGroup => Ok(self.build_blank_line_group(node)),
            _ => panic!("Unexpected node type"),
        }
    }

    fn build_paragraph(&self, node: ParseNode) -> ContentItem {
        let token_lines = group_tokens_by_line(node.tokens);
        ast_api::paragraph_from_token_lines(token_lines, self.source, &self.source_location)
    }

    fn build_session(&self, node: ParseNode) -> ParserResult<ContentItem> {
        let title_tokens = node.tokens;
        let content = self.build_session_content(node.children)?;
        Ok(ast_api::session_from_tokens(
            title_tokens,
            content,
            self.source,
            &self.source_location,
        ))
    }

    fn build_definition(&self, node: ParseNode) -> ParserResult<ContentItem> {
        let subject_tokens = node.tokens;
        let content = self.build_general_content(node.children, "Definition")?;
        Ok(ast_api::definition_from_tokens(
            subject_tokens,
            content,
            self.source,
            &self.source_location,
        ))
    }

    fn build_list(&self, node: ParseNode) -> ParserResult<ContentItem> {
        let list_items: Result<Vec<_>, _> = node
            .children
            .into_iter()
            .map(|child_node| self.build_list_item(child_node))
            .collect();
        Ok(ast_api::list_from_items(list_items?))
    }

    fn build_list_item(&self, node: ParseNode) -> ParserResult<ListItem> {
        let marker_tokens = node.tokens;
        let content = self.build_general_content(node.children, "ListItem")?;
        Ok(ast_api::list_item_from_tokens(
            marker_tokens,
            content,
            self.source,
            &self.source_location,
        ))
    }

    fn build_annotation(&self, node: ParseNode) -> ParserResult<ContentItem> {
        let header_tokens = node.tokens;
        let content = self.build_general_content(node.children, "Annotation")?;
        Ok(ast_api::annotation_from_tokens(
            header_tokens,
            content,
            self.source,
            &self.source_location,
        ))
    }

    fn build_verbatim_block(&self, mut node: ParseNode) -> ContentItem {
        let payload = node
            .payload
            .take()
            .expect("Parser must attach verbatim payload");
        let ParseNodePayload::VerbatimBlock {
            subject,
            content_lines,
            closing_data_tokens,
        } = payload;

        let closing_data =
            ast_api::data_from_tokens(closing_data_tokens, self.source, &self.source_location);

        ast_api::verbatim_block_from_lines(
            &subject,
            &content_lines,
            closing_data,
            self.source,
            &self.source_location,
        )
    }

    fn build_blank_line_group(&self, node: ParseNode) -> ContentItem {
        ast_api::blank_line_group_from_tokens(node.tokens, self.source, &self.source_location)
    }

    fn build_session_content(&self, nodes: Vec<ParseNode>) -> ParserResult<Vec<SessionContent>> {
        nodes
            .into_iter()
            .map(|node| self.build_content_item(node).map(SessionContent::from))
            .collect()
    }

    fn build_general_content(
        &self,
        nodes: Vec<ParseNode>,
        context: &str,
    ) -> ParserResult<Vec<ContentElement>> {
        nodes
            .into_iter()
            .map(|node| {
                self.build_content_item(node).and_then(|item| {
                    let location = item.range().clone();

                    // Extract text snippet from source for the invalid item (Session title)
                    // Get the line at the start of the error location
                    let source_lines: Vec<&str> = self.source.lines().collect();
                    let error_line_num = location.start.line;
                    let session_title = if error_line_num < source_lines.len() {
                        source_lines[error_line_num]
                    } else {
                        ""
                    };

                    ContentElement::try_from(item).map_err(|_| {
                        Box::new(ParserError::InvalidNesting {
                            container: context.to_string(),
                            invalid_child: "Session".to_string(),
                            invalid_child_text: session_title.to_string(),
                            location: location.clone(),
                            source_context: format_source_context(self.source, &location),
                        })
                    })
                })
            })
            .collect()
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
