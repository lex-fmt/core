Transform Pipeline Architecture

1. Overview

	This proposal describes a composable transformation pipeline architecture that replaces the previous complex config-driven pipeline system with a simpler, type-safe approach.

	The core principle: Everything is a Transform<Input, Output> that can be composed with other compatible transforms.

2. Problems with Previous Design

	The pipeline was simplified too much and lost critical capabilities:

	- No intermediate stage access: Cannot get tokens after core tokenization but before semantic indentation
	- No transformation configurability: Cannot apply transformations selectively or in custom orders
	- No custom pipeline composition: Cannot build custom pipelines like Markdown ’ Lex AST ’ Lex text
	- Testing inflexibility: Cannot inspect intermediate pipeline states

	Example of what was impossible:
		// Get tokens after JUST core tokenization
		let tokens = Lexplore::paragraph(1).base_tokens();

		// Apply custom transformation order
		let custom_pipeline = Pipeline::new()
		    .add_stage(core_tokenization)
		    .add_stage(custom_transform);
	:: rust ::

3. Core Design: Transform<I, O>

	3.1 The Transform Primitive

		Everything is a Transform with typed input and output:
			pub struct Transform<I, O> {
			    run_fn: Box<dyn Fn(I) -> Result<O, Error>>,
			}

			impl<I, O> Transform<I, O> {
			    fn new() -> Self { ... }

			    // Type-safe composition - compiler verifies compatibility
			    fn add<O2, S>(self, stage: S) -> Transform<I, O2>
			    where
			        S: Runnable<O, O2> + 'static
			    {
			        // Chains this.run() |> stage.run()
			        // Returns new Transform<I, O2>
			    }

			    fn run(&self, input: I) -> Result<O, Error> {
			        (self.run_fn)(input)
			    }
			}
		:: rust ::

	3.2 Composition Example

		Small transforms compose into larger ones:
			// Build lexing transform
			let lexing = Transform::new()
			    .add(CoreTokenization)       // String ’ TokenStream
			    .add(SemanticIndentation)    // TokenStream ’ TokenStream
			    .add(LineTokenGrouping);     // TokenStream ’ TokenStream
			// Result type: Transform<String, TokenStream>

			// Build parsing transform
			let parsing = Transform::new()
			    .add(SemanticAnalysis)       // TokenStream ’ IR[]
			    .add(AstBuilding);           // IR[] ’ Document
			// Result type: Transform<TokenStream, Document>

			// Compose into full pipeline
			let to_ast = Transform::new()
			    .add(lexing)                 // String ’ TokenStream
			    .add(parsing);               // TokenStream ’ Document
			// Result type: Transform<String, Document>

			// Use it
			let doc = to_ast.run(source)?;
		:: rust ::

	3.3 Type Safety

		The compiler enforces compatibility:
			let lexing: Transform<String, TokenStream> = ...;
			let parsing: Transform<TokenStream, Document> = ...;

			//  OK - types match
			let valid = Transform::new()
			    .add(lexing)    // String ’ TokenStream
			    .add(parsing);  // TokenStream ’ Document

			// L Compile error - type mismatch
			let invalid = Transform::new()
			    .add(parsing)   // String ’ ??? (expected TokenStream)
			    .add(lexing);
		:: rust ::

4. Standard Transforms

	4.1 Core Library Transforms

		Defined in lex-parser/src/lex/transforms.rs using lazy_static:
			lazy_static! {
			    // Lexing stages
			    pub static ref CORE_TOKENIZATION: Transform<String, TokenStream> =
			        Transform::new().add(CoreTokenization);

			    pub static ref LEXING: Transform<String, TokenStream> =
			        Transform::new()
			            .add(CoreTokenization)
			            .add(SemanticIndentation)
			            .add(LineTokenGrouping);

			    // Parsing stages
			    pub static ref PARSING: Transform<TokenStream, Document> =
			        Transform::new()
			            .add(SemanticAnalysis)
			            .add(AstBuilding);

			    // Full pipelines
			    pub static ref STRING_TO_AST: Transform<String, Document> =
			        Transform::new()
			            .add_transform(&*LEXING)
			            .add_transform(&*PARSING);
			}
		:: rust ::

	4.2 CLI-Specific Transforms

		Defined in lex-cli/src/transforms.rs:
			lazy_static! {
			    pub static ref AST_AS_TAG: Transform<String, String> =
			        Transform::new()
			            .add_transform(&*STRING_TO_AST)
			            .add(TagSerializer);

			    pub static ref AST_AS_JSON: Transform<String, String> =
			        Transform::new()
			            .add_transform(&*STRING_TO_AST)
			            .add(JsonSerializer);

			    pub static ref AST_AS_TREEVIZ: Transform<String, String> =
			        Transform::new()
			            .add_transform(&*STRING_TO_AST)
			            .add(TreeVizSerializer);
			}
		:: rust ::

5. DocumentLoader Integration

	5.1 Core Loader (lex-parser/src/lex/loader.rs)

		Provides path/string + transform shortcuts for ALL users, not just testing:
			pub struct DocumentLoader {
			    source: String,
			}

			impl DocumentLoader {
			    // Load from path
			    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
			        let source = fs::read_to_string(path)?;
			        Ok(DocumentLoader { source })
			    }

			    // Load from string
			    pub fn from_string(source: impl Into<String>) -> Self {
			        DocumentLoader { source: source.into() }
			    }

			    // Run any transform
			    pub fn with<O: 'static>(&self, transform: &Transform<String, O>) -> Result<O, Error> {
			        transform.run(&self.source)
			    }

			    // Shortcuts for common transforms
			    pub fn parse(&self) -> Result<Document, Error> {
			        self.with(&STRING_TO_AST)
			    }

			    pub fn tokenize(&self) -> Result<TokenStream, Error> {
			        self.with(&LEXING)
			    }

			    pub fn base_tokens(&self) -> Result<TokenStream, Error> {
			        self.with(&CORE_TOKENIZATION)
			    }

			    pub fn source(&self) -> &str {
			        &self.source
			    }
			}
		:: rust ::

	5.2 Testing Layer (lex-parser/src/lex/testing/lexplore/loader.rs)

		Adds magic file resolution on top of DocumentLoader:
			pub struct Lexplore;

			impl Lexplore {
			    // File resolution helpers
			    pub fn paragraph(number: usize) -> DocumentLoader {
			        let path = find_element_file(ElementType::Paragraph, number)
			            .expect("File not found");
			        DocumentLoader::from_path(path)
			            .expect("Failed to load file")
			    }

			    pub fn list(number: usize) -> DocumentLoader { ... }
			    pub fn session(number: usize) -> DocumentLoader { ... }
			    pub fn benchmark(number: usize) -> DocumentLoader { ... }
			    // etc...
			}
		:: rust ::

6. Usage Examples

	6.1 Production Usage (CLI, libraries, etc.)

		Using DocumentLoader directly:
			// From file path
			let doc = DocumentLoader::from_path("example.lex")?.parse()?;

			// From string
			let doc = DocumentLoader::from_string("Hello world\n").parse()?;

			// Custom transform
			let tokens = DocumentLoader::from_path("example.lex")?
			    .with(&CORE_TOKENIZATION)?;

			// CLI export
			let json = DocumentLoader::from_path("example.lex")?
			    .with(&CliTransforms::AST_AS_JSON)?;
		:: rust ::

	6.2 Test Usage

		Using Lexplore for magic file resolution:
			// One-liner tests
			#[test]
			fn test_paragraph() {
			    let doc = Lexplore::paragraph(1).parse().unwrap();
			    assert_eq!(doc.root.children.len(), 1);
			}

			// Custom transforms
			#[test]
			fn test_tokenization() {
			    let tokens = Lexplore::paragraph(1).base_tokens().unwrap();
			    assert!(tokens.len() > 0);
			}

			// Intermediate stages
			#[test]
			fn test_semantic_indentation() {
			    let tokens = Lexplore::session(1)
			        .with(&LEXING)
			        .unwrap();
			    // verify indentation handling
			}
		:: rust ::

	6.3 Custom Pipelines

		Building custom transformation pipelines:
			// Custom lexing pipeline
			let minimal_lexing = Transform::new()
			    .add(CoreTokenization)
			    .add(SemanticIndentation);
			    // Skip LineTokenGrouping

			// Use it
			let tokens = Lexplore::paragraph(1)
			    .with(&minimal_lexing)?;

			// Future: Markdown conversion
			let md_to_lex = Transform::new()
			    .add(MarkdownParser)      // String ’ MdAST
			    .add(MdToLexConverter)    // MdAST ’ LexAST
			    .add(LexSerializer);      // LexAST ’ String

			let lex_text = md_to_lex.run(markdown_source)?;
		:: rust ::

7. Implementation Plan

	Phase 1: Core Transform Infrastructure
		- Create lex/transforms/mod.rs with Transform<I, O> struct
		- Implement Runnable trait for individual stages
		- Add type-safe composition methods
		- Add error handling

	Phase 2: Standard Transforms
		- Define core transforms in lex/transforms/standard.rs
		- Migrate existing pipeline stages to implement Runnable
		- Create CORE_TOKENIZATION, LEXING, PARSING, STRING_TO_AST

	Phase 3: DocumentLoader
		- Create lex/loader.rs with DocumentLoader
		- Implement from_path, from_string, with, parse, tokenize
		- Add shortcuts for common transforms

	Phase 4: Testing Integration
		- Update Lexplore to return DocumentLoader
		- Update all tests to use new API
		- Remove old pipeline code

	Phase 5: CLI Integration
		- Create CLI-specific transforms
		- Update CLI to use DocumentLoader + transforms
		- Add format serialization transforms

8. Module Renaming

	Rename lex/pipeline to lex/transforms:
		lex/transforms/
		    mod.rs              # Transform<I,O> core
		    standard.rs         # Standard transform definitions
		    stages/             # Individual stage implementations
		        tokenization.rs
		        indentation.rs
		        parsing.rs
		        building.rs

	Keep lex/loader.rs separate (not in transforms, it's a utility).

9. Benefits

	 Composable: Small transforms build into larger ones
	 Type-safe: Compiler verifies transform compatibility
	 Reusable: Same transforms work in tests, CLI, libraries
	 Simple: One concept (Transform) does everything
	 Flexible: Support custom pipelines and intermediate stages
	 Testable: Can test individual transformation stages
	 No registry overhead: Static transforms, direct function calls
	 IDE-friendly: Autocomplete shows all available transforms

10. Non-Goals

	L Runtime transform discovery/loading
	L Plugin system for user-defined transforms
	L Dynamic configuration from files
	L Transform introspection/debugging (can add later if needed)

11. Migration Notes

	Old API:
		let doc = parse_document(source)?;
		let tokens = Lexplore::paragraph(1).tokenize();
	:: rust ::

	New API:
		let doc = DocumentLoader::from_string(source).parse()?;
		let tokens = Lexplore::paragraph(1).tokenize()?;
	:: rust ::

	The convenience function parse_document() can remain as a wrapper:
		pub fn parse_document(source: &str) -> Result<Document, Error> {
		    DocumentLoader::from_string(source).parse()
		}
	:: rust ::
