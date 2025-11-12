//! Simplified pipeline executor
use crate::lex::formats::{FormatError, FormatRegistry};
use crate::lex::parsing::Document;
use crate::lex::pipeline::Pipeline;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ExecutionError {
    ParsingFailed(String),
    FormatError(FormatError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::ParsingFailed(msg) => write!(f, "Parsing failed: {}", msg),
            ExecutionError::FormatError(err) => write!(f, "Format error: {}", err),
        }
    }
}

impl std::error::Error for ExecutionError {}

pub enum ExecutionOutput {
    Document(Document),
    Serialized(String),
}

pub struct PipelineExecutor {
    format_registry: FormatRegistry,
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            format_registry: FormatRegistry::with_defaults(),
        }
    }

    pub fn execute(&self, source: &str) -> Result<ExecutionOutput, ExecutionError> {
        let pipeline = Pipeline::new();
        let doc = pipeline.run(source).map_err(ExecutionError::ParsingFailed)?;
        Ok(ExecutionOutput::Document(doc))
    }

    pub fn execute_and_serialize(
        &self,
        source: &str,
        format: &str,
    ) -> Result<String, ExecutionError> {
        let output = self.execute(source)?;
        match output {
            ExecutionOutput::Document(doc) => self
                .format_registry
                .serialize(&doc, format)
                .map_err(ExecutionError::FormatError),
            ExecutionOutput::Serialized(s) => Ok(s),
        }
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}
