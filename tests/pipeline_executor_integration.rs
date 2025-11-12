//! Integration tests for PipelineExecutor
//!
//! These tests validate that the config-based executor:
//! 1. Produces correct results on real sample files
//! 2. All default configs work without errors

use lex::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use lex::lex::testing::lexplore::{DocumentType, Lexplore};

#[derive(Copy, Clone)]
enum SampleSpec {
    Trifecta {
        number: usize,
        label: &'static str,
    },
}

impl SampleSpec {
    fn load(&self) -> String {
        match *self {
            SampleSpec::Trifecta { number, .. } => {
                Lexplore::load_document(DocumentType::Trifecta, number).source()
            }
        }
    }

    fn label(&self) -> &'static str {
        match *self {
            SampleSpec::Trifecta { label, .. } => label,
        }
    }
}

#[test]
fn test_executor_on_sample_paragraphs() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::trifecta(0).source();

    let result = executor.execute(&source);
    assert!(
        result.is_ok(),
        "Failed to parse trifecta-000-paragraphs document"
    );

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(
                !doc.root.children.is_empty(),
                "Document should have content"
            );
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_executor_on_sample_lists() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::trifecta(40).source();

    let result = executor.execute(&source);
    assert!(
        result.is_ok(),
        "Failed to parse trifecta-040-lists document"
    );

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_executor_on_sample_definitions() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::definition(1).source();

    let result = executor.execute(&source);
    assert!(
        result.is_ok(),
        "Failed to parse definition-01-flat-simple element document"
    );

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_executor_on_sample_sessions() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::trifecta(10).source();

    let result = executor.execute(&source);
    assert!(result.is_ok());

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_executor_simple_source() {
    let executor = PipelineExecutor::new();
    let source = "Hello world\n";

    let result = executor.execute(source).unwrap();
    let doc = match result {
        ExecutionOutput::Document(doc) => doc,
        _ => panic!("Expected document"),
    };

    // Should produce document with content
    assert!(
        !doc.root.children.is_empty(),
        "Document should have content"
    );
}

#[test]
fn test_executor_with_session() {
    let executor = PipelineExecutor::new();
    let source = "Session:\n    Content here\n";

    let result = executor.execute(source).unwrap();
    let doc = match result {
        ExecutionOutput::Document(doc) => doc,
        _ => panic!("Expected document"),
    };

    // Should produce document with session
    assert!(
        !doc.root.children.is_empty(),
        "Document should have session content"
    );
}

#[test]
fn test_default_config_works() {
    let executor = PipelineExecutor::new();
    let source = "Test:\n    Content\n";

    // The default config should work without error
    let result = executor.execute(source);
    assert!(
        result.is_ok(),
        "Default config failed: {:?}",
        result.err()
    );
}

#[test]
fn test_on_multiple_samples() {
    let executor = PipelineExecutor::new();

    const SAMPLE_SPECS: &[SampleSpec] = &[
        SampleSpec::Trifecta {
            number: 0,
            label: "trifecta-000-paragraphs",
        },
        SampleSpec::Trifecta {
            number: 40,
            label: "trifecta-040-lists",
        },
    ];

    for sample in SAMPLE_SPECS {
        let source = sample.load();

        let output = executor
            .execute(&source)
            .unwrap_or_else(|err| {
                panic!(
                    "Default config failed on {}: {:?}",
                    sample.label(),
                    err
                )
            });

        match output {
            ExecutionOutput::Document(doc) => {
                assert!(
                    !doc.root.children.is_empty(),
                    "Document from {} should have content",
                    sample.label()
                );
            }
            _ => panic!("Expected Document from sample '{}'", sample.label()),
        }
    }
}


#[test]
fn test_executor_with_nested_content() {
    let executor = PipelineExecutor::new();
    let source = "Outer:\n    Inner:\n        Deep content\n";

    let result = executor.execute(source);
    assert!(result.is_ok(), "Should handle nested content");

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}
