Containers , Structure and Parsing in Lex

	Lex is a structurally recursive language, where almost all elements can contain children. This structure gives Lex a powerful primitive, giving users a much richer representation of their data then most flat formats like HTML or markdown.

	All parent -> child relationships are denoted by indentation. From the AST perspective, this means a container node. The container stack is what the parser manipulates when indentation changes.

1. Container Policies

	Containers now encode their nesting rules at the type level. Each container alias wraps a `ContainerPolicy`:
		SessionContainer → `SessionContent` (sessions + general elements)
		GeneralContainer → `ContentElement` (no sessions)
		ListContainer → `ListContent` (only list items)
		VerbatimContainer → `VerbatimContent` (only verbatim lines)

	See :: file src/lex/ast/elements/container.rs :: for the policy definitions. The typed wrappers live in :: file src/lex/ast/elements/typed_content.rs ::

2. Constructors Always Take Typed Vecs

	Every AST constructor now reflects the policy at the signature level:
		Session::new(title, Vec<SessionContent>)
		Definition::new(subject, Vec<ContentElement>)
		Annotation::new(label, params, Vec<ContentElement>)
		List::new(Vec<ListItem>) (each ListItem already owns a GeneralContainer)
		Verbatim::new(subject, Vec<VerbatimContent>, closing_annotation)

	Consequences:
		No more `ContainerNode`, `Container::new`, or `_from_text` helpers.
		If a call site compiles, the nesting rules hold.
		Runtime `expect("cannot contain")` checks were deleted.

3. Parser / Builder Flow

	`AstTreeBuilder` converts `ParseNode` children into typed vectors before calling the builder API. Example (see :: file src/lex/parsing/builder.rs ::):
		Sessions → `session_from_tokens(title, Vec<SessionContent>, source)`
		Definitions / annotations / list items → `*_from_tokens` with `Vec<ContentElement>`

	Builders aggregate locations by temporarily projecting the typed vec back into `ContentItem`, but the AST structs only ever see the typed inputs.

4. Compile-Fail Tests

	Trybuild tests ensure regressions are caught at compile time. The harness lives in :: file tests/compile_fail.rs :: and the cases live under :: dir tests/compile_fail ::

	Example (definition rejecting sessions):
	```lex
	:: file tests/compile_fail/definition_rejects_session.rs ::
		let _definition = Definition::new(subject, vec![SessionContent::Session(session)]);
	```

	Running `TRYBUILD=overwrite cargo test -p lex-parser --test compile_fail` should only be necessary when intentionally changing diagnostics.

5. Authoring Guidelines

	When adding a new container element:
		Define an appropriate policy in container.rs or reuse General/Session policy.
		Create a typed enum variant under `typed_content.rs` if the container needs special rules.
		Thread the typed vector through the builder and parser entry points.
		Add a compile-fail test if the new container forbids certain children.

	When touching existing containers:
		Avoid reintroducing `Vec<ContentItem>` parameters.
		Prefer `Container::<Policy>::from_typed` when constructing containers directly.
		Keep traversal code (`ContentItem`, visitors, viewer UI) in terms of the enum—typed content is for construction, not tree walking.

6. Further Reading

	Architecture overview: :: file docs/architecture/type-safe-containers.md ::
	Public crate README: :: file lex-parser/README.md ::
	Compile-fail harness: :: file lex-parser/tests/compile_fail.rs ::

	These documents explain the rationale behind issues #228 → #235 and describe how the type-safe container pipeline ties the parser, builders, and AST together.
