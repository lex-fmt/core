//! Line Token Grouping Transformation
//!
//! Groups flat tokens into line-based groups with classification.
//! This transformation:
//! - Groups consecutive tokens into lines (delimited by Newline)
//! - Classifies each line by type (SubjectLine, ListLine, etc.)
//! - Handles structural tokens (Indent, Dedent, BlankLine) specially
//! - Applies dialog line detection
//!
//! Converts: TokenStream::Flat → TokenStream::Grouped

use crate::lex::lexing::line_grouping::group_into_lines;
use crate::lex::lexing::tokens_core::Token;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::{GroupType, GroupedTokens, TokenStream};
use std::ops::Range as ByteRange;

/// Transformation that groups flat tokens into line-based groups.
pub struct LineTokenGroupingMapper;

impl LineTokenGroupingMapper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineTokenGroupingMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMapper for LineTokenGroupingMapper {
    fn map_flat(
        &mut self,
        tokens: Vec<(Token, ByteRange<usize>)>,
    ) -> Result<TokenStream, TransformationError> {
        // Group tokens into LineTokens
        let line_tokens = group_into_lines(tokens);

        // Convert LineTokens to GroupedTokens
        let grouped_tokens: Vec<GroupedTokens> = line_tokens
            .into_iter()
            .map(|line_token| GroupedTokens {
                source_tokens: line_token.source_token_pairs(),
                group_type: GroupType::Line(line_token.line_type),
            })
            .collect();

        Ok(TokenStream::Grouped(grouped_tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexing::tokens_linebased::LineType;
    use crate::lex::pipeline::mapper::StreamMapper;

    #[test]
    fn test_mapper_integration() {
        let tokens = vec![
            (Token::Text("Title".to_string()), 0..5),
            (Token::Colon, 5..6),
            (Token::Newline, 6..7),
        ];

        let mut mapper = LineTokenGroupingMapper::new();
        let result = mapper.map_flat(tokens).unwrap();

        match result {
            TokenStream::Grouped(groups) => {
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].source_tokens.len(), 3);
                match groups[0].group_type {
                    GroupType::Line(LineType::SubjectLine) => {}
                    _ => panic!("Expected SubjectLine"),
                }
            }
            _ => panic!("Expected Grouped stream"),
        }
    }
}
