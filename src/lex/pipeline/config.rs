//! Processing configuration system for Lex pipelines
//!
//! This module defines named configurations that specify:
//! 1. Which transformation pipeline to run (e.g., Indentation, Linebased)
//! 2. Whether to stop at tokens or continue to AST
//! 3. Which parser to use (if continuing to AST)

use std::collections::HashMap;

/// A named configuration specifying transformation pipeline and target
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub name: String,
    pub description: String,
    pub pipeline_spec: PipelineSpec,
    pub target: TargetSpec,
}

/// Which transformation pipeline to use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineSpec {
    /// Standard indentation-based transformations
    /// [NormalizeWhitespace, SemanticIndentation, BlankLines]
    Indentation,

    /// Linebased transformations (full, with tree)
    /// [NormalizeWhitespace, SemanticIndentation, BlankLines, ToLineTokens, IndentationToTree]
    Linebased,

    /// Linebased up to LineTokens (no tree)
    /// [NormalizeWhitespace, SemanticIndentation, BlankLines, ToLineTokens]
    LinebasedFlat,

    /// Raw tokens only (minimal transformations)
    Raw,
}

/// What to produce from the pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetSpec {
    /// Stop at tokens
    Tokens,

    /// Continue to AST with specified analyzer and builder
    Ast {
        analyzer: AnalysisSpec,
        builder: BuilderSpec,
    },

    /// Continue to AST and serialize to specified format
    Serialized {
        analyzer: AnalysisSpec,
        builder: BuilderSpec,
        format: String,
    },
}

/// Which syntactic analyzer (parser) to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisSpec {
    /// Reference combinator parser
    Reference,

    /// Linebased declarative grammar parser
    Linebased,
}

/// Which AST builder to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderSpec {
    /// Standard LSP AST builder
    Lsp,
}

/// Registry of processing configurations
pub struct ConfigRegistry {
    configs: HashMap<String, ProcessingConfig>,
}

impl ConfigRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        ConfigRegistry {
            configs: HashMap::new(),
        }
    }

    /// Register a configuration
    pub fn register(&mut self, config: ProcessingConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    /// Get a configuration by name
    pub fn get(&self, name: &str) -> Option<&ProcessingConfig> {
        self.configs.get(name)
    }

    /// Check if a configuration exists
    pub fn has(&self, name: &str) -> bool {
        self.configs.contains_key(name)
    }

    /// List all configurations (sorted by name)
    pub fn list_all(&self) -> Vec<&ProcessingConfig> {
        let mut configs: Vec<_> = self.configs.values().collect();
        configs.sort_by(|a, b| a.name.cmp(&b.name));
        configs
    }

    /// Create registry with standard configurations
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Standard stable configuration
        registry.register(ProcessingConfig {
            name: "default".into(),
            description: "Stable: Indentation lexer + Reference analyzer + LSP builder".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Ast {
                analyzer: AnalysisSpec::Reference,
                builder: BuilderSpec::Lsp,
            },
        });

        // Linebased experimental configuration
        registry.register(ProcessingConfig {
            name: "linebased".into(),
            description: "Experimental: Linebased lexer + Linebased analyzer + LSP builder".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Ast {
                analyzer: AnalysisSpec::Linebased,
                builder: BuilderSpec::Lsp,
            },
        });

        // Token-only configurations (for testing/debugging)
        registry.register(ProcessingConfig {
            name: "tokens-indentation".into(),
            description: "Indentation transformations, output tokens".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-flat".into(),
            description: "Linebased up to LineTokens, output tokens".into(),
            pipeline_spec: PipelineSpec::LinebasedFlat,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-tree".into(),
            description: "Full linebased with tree, output tokens".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-raw".into(),
            description: "Raw tokens from base tokenization".into(),
            pipeline_spec: PipelineSpec::Raw,
            target: TargetSpec::Tokens,
        });

        // Serialization configurations
        registry.register(ProcessingConfig {
            name: "lex-to-tag".into(),
            description: "Parse and serialize to tag format (AST XML-like)".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Serialized {
                analyzer: AnalysisSpec::Reference,
                builder: BuilderSpec::Lsp,
                format: "tag".into(),
            },
        });

        registry.register(ProcessingConfig {
            name: "lex-to-treeviz".into(),
            description: "Parse and serialize to treeviz format (tree visualization)".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Serialized {
                analyzer: AnalysisSpec::Reference,
                builder: BuilderSpec::Lsp,
                format: "treeviz".into(),
            },
        });

        registry.register(ProcessingConfig {
            name: "lex-to-tag-linebased".into(),
            description: "Parse with linebased and serialize to tag format".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Serialized {
                analyzer: AnalysisSpec::Linebased,
                builder: BuilderSpec::Lsp,
                format: "tag".into(),
            },
        });

        registry.register(ProcessingConfig {
            name: "lex-to-treeviz-linebased".into(),
            description: "Parse with linebased and serialize to treeviz format".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Serialized {
                analyzer: AnalysisSpec::Linebased,
                builder: BuilderSpec::Lsp,
                format: "treeviz".into(),
            },
        });

        registry
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ConfigRegistry::new();
        assert_eq!(registry.configs.len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ConfigRegistry::new();
        let config = ProcessingConfig {
            name: "test".into(),
            description: "Test config".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Tokens,
        };

        registry.register(config);
        assert_eq!(registry.configs.len(), 1);
        assert!(registry.has("test"));
    }

    #[test]
    fn test_registry_get() {
        let registry = ConfigRegistry::with_defaults();
        let config = registry.get("default").unwrap();
        assert_eq!(config.name, "default");
        assert_eq!(config.pipeline_spec, PipelineSpec::Indentation);
        assert!(matches!(config.target, TargetSpec::Ast { .. }));
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = ConfigRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_has() {
        let registry = ConfigRegistry::with_defaults();
        assert!(registry.has("default"));
        assert!(registry.has("linebased"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_registry_list_all() {
        let registry = ConfigRegistry::with_defaults();
        let configs = registry.list_all();

        assert!(configs.len() >= 6); // Should have at least 6 default configs

        // Verify sorted by name
        for i in 1..configs.len() {
            assert!(configs[i - 1].name <= configs[i].name);
        }
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ConfigRegistry::with_defaults();

        // Check all expected configs exist
        assert!(registry.get("default").is_some());
        assert!(registry.get("linebased").is_some());
        assert!(registry.get("tokens-indentation").is_some());
        assert!(registry.get("tokens-linebased-flat").is_some());
        assert!(registry.get("tokens-linebased-tree").is_some());
        assert!(registry.get("tokens-raw").is_some());
    }

    #[test]
    fn test_registry_default_trait() {
        let registry = ConfigRegistry::default();
        assert!(registry.has("default"));
    }

    #[test]
    fn test_default_config_details() {
        let registry = ConfigRegistry::with_defaults();
        let config = registry.get("default").unwrap();

        assert_eq!(config.name, "default");
        assert!(config.description.contains("Stable"));
        assert_eq!(config.pipeline_spec, PipelineSpec::Indentation);
        assert_eq!(
            config.target,
            TargetSpec::Ast {
                analyzer: AnalysisSpec::Reference,
                builder: BuilderSpec::Lsp,
            }
        );
    }

    #[test]
    fn test_linebased_config_details() {
        let registry = ConfigRegistry::with_defaults();
        let config = registry.get("linebased").unwrap();

        assert_eq!(config.name, "linebased");
        assert!(config.description.contains("Experimental"));
        assert_eq!(config.pipeline_spec, PipelineSpec::Linebased);
        assert_eq!(
            config.target,
            TargetSpec::Ast {
                analyzer: AnalysisSpec::Linebased,
                builder: BuilderSpec::Lsp,
            }
        );
    }

    #[test]
    fn test_tokens_config_details() {
        let registry = ConfigRegistry::with_defaults();
        let config = registry.get("tokens-indentation").unwrap();

        assert_eq!(config.pipeline_spec, PipelineSpec::Indentation);
        assert_eq!(config.target, TargetSpec::Tokens);
    }

    #[test]
    fn test_pipeline_spec_equality() {
        assert_eq!(PipelineSpec::Indentation, PipelineSpec::Indentation);
        assert_ne!(PipelineSpec::Indentation, PipelineSpec::Linebased);
    }

    #[test]
    fn test_target_spec_equality() {
        assert_eq!(TargetSpec::Tokens, TargetSpec::Tokens);
        assert_ne!(
            TargetSpec::Tokens,
            TargetSpec::Ast {
                analyzer: AnalysisSpec::Reference,
                builder: BuilderSpec::Lsp,
            }
        );
    }

    #[test]
    fn test_analyzer_spec_equality() {
        assert_eq!(AnalysisSpec::Reference, AnalysisSpec::Reference);
        assert_ne!(AnalysisSpec::Reference, AnalysisSpec::Linebased);
    }

    #[test]
    fn test_builder_spec_equality() {
        assert_eq!(BuilderSpec::Lsp, BuilderSpec::Lsp);
    }
}
