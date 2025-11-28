use crate::error::FormatError;
use crate::format::SerializedDocument;
use crate::registry::FormatRegistry;
use lex_parser::lex::ast::Document;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PublishSpec<'a> {
    pub document: &'a Document,
    pub format: &'a str,
    pub output: Option<PathBuf>,
    pub options: HashMap<String, String>,
}

impl<'a> PublishSpec<'a> {
    pub fn new(document: &'a Document, format: &'a str) -> Self {
        Self {
            document,
            format,
            output: None,
            options: HashMap::new(),
        }
    }

    pub fn with_output_path(mut self, path: impl AsRef<Path>) -> Self {
        self.output = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PublishArtifact {
    InMemory(String),
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishResult {
    pub artifact: PublishArtifact,
}

pub fn publish(spec: PublishSpec<'_>) -> Result<PublishResult, FormatError> {
    let registry = FormatRegistry::with_defaults();
    let serialized = registry.serialize_with_options(spec.document, spec.format, &spec.options)?;
    match serialized {
        SerializedDocument::Text(text) => write_or_return_text(text, spec.output),
        SerializedDocument::Binary(bytes) => write_binary(bytes, spec.output),
    }
}

fn write_or_return_text(
    text: String,
    output: Option<PathBuf>,
) -> Result<PublishResult, FormatError> {
    if let Some(path) = output {
        write_to_path(path, text.into_bytes()).map(|path| PublishResult {
            artifact: PublishArtifact::File(path),
        })
    } else {
        Ok(PublishResult {
            artifact: PublishArtifact::InMemory(text),
        })
    }
}

fn write_binary(bytes: Vec<u8>, output: Option<PathBuf>) -> Result<PublishResult, FormatError> {
    let path = output.ok_or_else(|| {
        FormatError::SerializationError(
            "binary formats require an explicit output path".to_string(),
        )
    })?;
    write_to_path(path, bytes).map(|path| PublishResult {
        artifact: PublishArtifact::File(path),
    })
}

fn write_to_path(path: PathBuf, bytes: Vec<u8>) -> Result<PathBuf, FormatError> {
    fs::write(&path, &bytes)
        .map(|_| path.clone())
        .map_err(|err| FormatError::SerializationError(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::parsing;
    use tempfile::tempdir;

    const SAMPLE: &str = "Title:\n\nParagraph text.\n";

    fn sample_document() -> Document {
        parsing::parse_document(SAMPLE).unwrap()
    }

    #[test]
    fn publishes_to_memory_when_no_output_path() {
        let doc = sample_document();
        let result = publish(PublishSpec::new(&doc, "html")).expect("publish");
        match result.artifact {
            PublishArtifact::InMemory(content) => {
                assert!(content.contains("Paragraph text."));
            }
            PublishArtifact::File(_) => panic!("expected in-memory artifact"),
        }
    }

    #[test]
    fn writes_to_disk_when_output_path_provided() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.html");
        let doc = sample_document();
        let result =
            publish(PublishSpec::new(&doc, "html").with_output_path(&path)).expect("publish");
        match result.artifact {
            PublishArtifact::File(p) => assert_eq!(p, path),
            PublishArtifact::InMemory(_) => panic!("expected file artifact"),
        }
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("Paragraph text."));
    }
}
