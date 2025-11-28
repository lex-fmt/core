//! Shared configuration loader for the Lex toolchain.
//!
//! `defaults/lex.default.toml` is embedded into every binary so that docs and
//! runtime behavior stay in sync. Applications layer user-specific files on top
//! of those defaults via [`Loader`] before deserializing into [`LexConfig`].

use config::builder::DefaultState;
use config::{Config, ConfigBuilder, ConfigError, File, FileFormat, ValueKind};
use serde::Deserialize;
use std::path::Path;

const DEFAULT_TOML: &str = include_str!("../defaults/lex.default.toml");

/// Top-level configuration consumed by Lex applications.
#[derive(Debug, Clone, Deserialize)]
pub struct LexConfig {
    pub formatting: FormattingConfig,
    pub inspect: InspectConfig,
    pub convert: ConvertConfig,
}

/// Formatting-related configuration groups.
#[derive(Debug, Clone, Deserialize)]
pub struct FormattingConfig {
    pub rules: FormattingRulesConfig,
}

/// Mirrors the knobs exposed by the Lex formatter.
#[derive(Debug, Clone, Deserialize)]
pub struct FormattingRulesConfig {
    pub session_blank_lines_before: usize,
    pub session_blank_lines_after: usize,
    pub normalize_seq_markers: bool,
    pub unordered_seq_marker: char,
    pub max_blank_lines: usize,
    pub indent_string: String,
    pub preserve_trailing_blanks: bool,
    pub normalize_verbatim_markers: bool,
}

/// Controls AST-related inspect output.
#[derive(Debug, Clone, Deserialize)]
pub struct InspectConfig {
    pub ast: InspectAstConfig,
    pub nodemap: NodemapConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InspectAstConfig {
    pub include_all_properties: bool,
    pub show_line_numbers: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodemapConfig {
    pub color_blocks: bool,
    pub color_characters: bool,
    pub show_summary: bool,
}

/// Format-specific conversion knobs.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertConfig {
    pub pdf: PdfConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PdfConfig {
    pub size: PdfPageSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PdfPageSize {
    Desktop,
    Mobile,
}

/// Helper for layering user overrides over the built-in defaults.
#[derive(Debug, Clone)]
pub struct Loader {
    builder: ConfigBuilder<DefaultState>,
}

impl Loader {
    /// Start a loader seeded with the embedded defaults.
    pub fn new() -> Self {
        let builder = Config::builder().add_source(File::from_str(DEFAULT_TOML, FileFormat::Toml));
        Self { builder }
    }

    /// Layer a configuration file. Missing files trigger an error.
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Self {
        let source = File::from(path.as_ref())
            .format(FileFormat::Toml)
            .required(true);
        self.builder = self.builder.add_source(source);
        self
    }

    /// Layer an optional configuration file (ignored if the file is absent).
    pub fn with_optional_file(mut self, path: impl AsRef<Path>) -> Self {
        let source = File::from(path.as_ref())
            .format(FileFormat::Toml)
            .required(false);
        self.builder = self.builder.add_source(source);
        self
    }

    /// Apply a single key/value override (useful for CLI settings).
    pub fn set_override<I>(mut self, key: &str, value: I) -> Result<Self, ConfigError>
    where
        I: Into<ValueKind>,
    {
        self.builder = self.builder.set_override(key, value)?;
        Ok(self)
    }

    /// Finalize the builder and deserialize the resulting configuration.
    pub fn build(self) -> Result<LexConfig, ConfigError> {
        self.builder.build()?.try_deserialize()
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience helper for callers that only need the defaults.
pub fn load_defaults() -> Result<LexConfig, ConfigError> {
    Loader::new().build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_default_config() {
        let config = load_defaults().expect("defaults to deserialize");
        assert_eq!(config.formatting.rules.session_blank_lines_before, 1);
        assert!(config.inspect.ast.show_line_numbers);
        assert_eq!(config.convert.pdf.size, PdfPageSize::Desktop);
    }

    #[test]
    fn supports_overrides() {
        let config = Loader::new()
            .set_override("convert.pdf.size", "mobile")
            .expect("override to apply")
            .build()
            .expect("config to build");
        assert_eq!(config.convert.pdf.size, PdfPageSize::Mobile);
    }
}
