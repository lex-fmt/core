//! Integration tests for PipelineExecutor
//!
//! These tests validate that the config-based executor:
//! 1. Produces correct results on real sample files
//! 2. All default configs work without errors

use lex::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use lex::lex::processor::lex_sources::LexSources;

#[test]
fn test_executor_on_sample_paragraphs() {
    let executor = PipelineExecutor::new();
    let source = LexSources::get_string("000-paragraphs.lex").unwrap();

    let result = executor.execute("default", &source);
    assert!(result.is_ok(), "Failed to parse 000-paragraphs.lex");

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
    let source = LexSources::get_string("040-lists.lex").unwrap();

    let result = executor.execute("default", &source);
    assert!(result.is_ok(), "Failed to parse 040-lists.lex");

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
    let source = LexSources::get_string("090-definitions-simple.lex").unwrap();

    let result = executor.execute("default", &source);
    assert!(result.is_ok(), "Failed to parse 090-definitions-simple.lex");

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
    let source = LexSources::get_string("010-paragraphs-sessions-flat-single.lex").unwrap();

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

    let samples = vec![
        "000-paragraphs.lex",
        "040-lists.lex",
        "090-definitions-simple.lex",
    ];

    for sample in samples {
        let source = LexSources::get_string(sample).unwrap();

        // Test configs that produce AST
        for config_name in &["default", "linebased"] {
            let result = executor.execute(config_name, &source);
            assert!(
                result.is_ok(),
                "Config '{}' failed on {}: {:?}",
                config_name,
                sample,
                result.err()
            );

            match result.unwrap() {
                ExecutionOutput::Document(doc) => {
                    assert!(
                        !doc.root.children.is_empty(),
                        "Document from '{}' on {} should have content",
                        config_name,
                        sample
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
    let source = LexSources::get_string("000-paragraphs.lex").unwrap();

    let result = executor.execute("linebased", &source);
    assert!(result.is_ok(), "Linebased config should work on sample");

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
        lex::lex::pipeline::ExecutionError::ConfigNotFound(name) => {
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
