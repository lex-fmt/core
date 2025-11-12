//! Simplified processing pipeline for the lex format
use crate::lex::parsing::Document;

pub mod executor;
pub use executor::{ExecutionError, ExecutionOutput, PipelineExecutor};

pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, source: &str) -> Result<Document, String> {
        let source_with_newline = crate::lex::lexing::ensure_source_ends_with_newline(source);
        let token_stream = crate::lex::lexing::base_tokenization::tokenize(&source_with_newline);
        let tokens = crate::lex::lexing::lex(token_stream);
        crate::lex::parsing::linebased::parse_from_flat_tokens(tokens, source)
            .map_err(|err| format!("Parsing failed: {}", err))
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
