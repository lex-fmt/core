//! Integration tests for PipelineExecutor
//!
//! These tests validate that the config-based executor:
//! 1. Produces correct results on real sample files
//! 2. All default configs work without errors

use lex_parser::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use lex_parser::lex::testing::lexplore::{DocumentType, ElementType, Lexplore};

#[derive(Copy, Clone)]
enum SampleSpec {
    Trifecta {
        number: usize,
        label: &'static str,
    },
    Element {
        element: ElementType,
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
            SampleSpec::Element {
                element, number, ..
            } => Lexplore::load(element, number).source(),
        }
    }

    fn label(&self) -> &'static str {
        match *self {
            SampleSpec::Trifecta { label, .. } | SampleSpec::Element { label, .. } => label,
        }
    }
}

#[test]
fn test_executor_on_sample_paragraphs() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::trifecta(0).source();

    let result = executor.execute("default", &source);
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

    let result = executor.execute("default", &source);
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

    let result = executor.execute("default", &source);
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

    let result = executor.execute("default", &source);
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

    let result = executor.execute("default", source).unwrap();
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

    let result = executor.execute("default", source).unwrap();
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
fn test_all_default_configs_work() {
    let executor = PipelineExecutor::new();
    let source = "Test:\n    Content\n";

    // Every default config should work without error
    for config in executor.list_configs() {
        let result = executor.execute(&config.name, source);
        assert!(
            result.is_ok(),
            "Config '{}' failed: {:?}",
            config.name,
            result.err()
        );
    }
}

#[test]
fn test_all_configs_on_multiple_samples() {
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
        SampleSpec::Element {
            element: ElementType::Definition,
            number: 1,
            label: "definition-01-flat-simple",
        },
    ];

    for sample in SAMPLE_SPECS {
        let source = sample.load();

        // Test configs that produce AST
        for config_name in &["default", "linebased"] {
            let output = executor
                .execute(config_name, &source)
                .unwrap_or_else(|err| {
                    panic!(
                        "Config '{}' failed on {}: {:?}",
                        config_name,
                        sample.label(),
                        err
                    )
                });

            match output {
                ExecutionOutput::Document(doc) => {
                    assert!(
                        !doc.root.children.is_empty(),
                        "Document from '{}' on {} should have content",
                        config_name,
                        sample.label()
                    );
                }
                _ => panic!("Expected Document from config '{}'", config_name),
            }
        }
    }
}

#[test]
fn test_token_output_configs() {
    let executor = PipelineExecutor::new();
    let source = "Hello world\n";

    let token_configs = vec![
        "tokens-indentation",
        "tokens-linebased-flat",
        "tokens-linebased-tree",
        "tokens-raw",
    ];

    for config_name in token_configs {
        let result = executor.execute(config_name, source);
        assert!(result.is_ok(), "Token config '{}' failed", config_name);

        match result.unwrap() {
            ExecutionOutput::Tokens(stream) => {
                let tokens = stream.unroll();
                assert!(
                    !tokens.is_empty(),
                    "Config '{}' should produce tokens",
                    config_name
                );
            }
            _ => panic!("Config '{}' should return Tokens", config_name),
        }
    }
}

#[test]
fn test_linebased_config_on_sample() {
    let executor = PipelineExecutor::new();
    let source = Lexplore::trifecta(0).source();

    let result = executor.execute("linebased", &source);
    assert!(
        result.is_ok(),
        "Linebased config should work on trifecta-000-paragraphs"
    );

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_executor_minimal_source() {
    let executor = PipelineExecutor::new();
    let source = "x\n";

    let result = executor.execute("default", source);
    assert!(result.is_ok(), "Should handle minimal source");

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}

#[test]
fn test_config_registry_has_expected_configs() {
    let executor = PipelineExecutor::new();
    let registry = executor.registry();

    // Check all expected configs exist
    assert!(registry.has("default"));
    assert!(registry.has("linebased"));
    assert!(registry.has("tokens-indentation"));
    assert!(registry.has("tokens-linebased-flat"));
    assert!(registry.has("tokens-linebased-tree"));
    assert!(registry.has("tokens-raw"));
}

#[test]
fn test_nonexistent_config_error() {
    let executor = PipelineExecutor::new();
    let source = "test\n";

    let result = executor.execute("nonexistent-config", source);
    assert!(result.is_err());

    match result.unwrap_err() {
        lex_parser::lex::pipeline::ExecutionError::ConfigNotFound(name) => {
            assert_eq!(name, "nonexistent-config");
        }
        _ => panic!("Expected ConfigNotFound error"),
    }
}

#[test]
fn test_tokens_raw_produces_minimal_tokens() {
    let executor = PipelineExecutor::new();
    let source = "Hello world\n";

    let result = executor.execute("tokens-raw", source);
    assert!(result.is_ok());

    match result.unwrap() {
        ExecutionOutput::Tokens(stream) => {
            let tokens = stream.unroll();
            // Raw tokens should have basic tokens without much transformation
            assert!(!tokens.is_empty());
        }
        _ => panic!("Expected Tokens output"),
    }
}

#[test]
fn test_executor_with_nested_content() {
    let executor = PipelineExecutor::new();
    let source = "Outer:\n    Inner:\n        Deep content\n";

    let result = executor.execute("default", source);
    assert!(result.is_ok(), "Should handle nested content");

    match result.unwrap() {
        ExecutionOutput::Document(doc) => {
            assert!(!doc.root.children.is_empty());
        }
        _ => panic!("Expected document output"),
    }
}
