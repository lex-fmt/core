# Type-Safe Container System

The container refactor from issues #228–#235 replaced the old "bag of `ContentItem`"
representation with policy-driven containers. This document captures the mental model so
future contributors can extend the AST without re-introducing runtime validation.

## Policy Basics

Each container in `lex_parser::lex::ast::elements::container` is backed by a
`ContainerPolicy` implementation:

| Policy | Alias | Accepted children | Used by |
| --- | --- | --- | --- |
| `SessionPolicy` | `SessionContainer` | `SessionContent` (sessions and general elements) | `Document.root`, `Session.children` |
| `GeneralPolicy` | `GeneralContainer` | `ContentElement` (no sessions) | `Definition`, `Annotation`, `ListItem` |
| `ListPolicy` | `ListContainer` | `ListContent` (only list items) | `List.items` |
| `VerbatimPolicy` | `VerbatimContainer` | `VerbatimContent` (only verbatim lines) | `Verbatim.children`, verbatim groups |

Every AST constructor now accepts the typed vector for its container. If code compiles,
then the container invariants are satisfied—no runtime `validate_*` calls remain.

## Using the Typed APIs

```rust
use lex_parser::lex::ast::elements::typed_content::ContentElement;
use lex_parser::lex::ast::{Definition, TextContent};

let subject = TextContent::from_string("Term".into(), None);
let children: Vec<ContentElement> = vec![]; // paragraphs, lists, annotations, ...
let def = Definition::new(subject, children);
```

`Session::new` follows the same pattern but accepts `Vec<SessionContent>`, so sessions can
nest arbitrarily while still permitting paragraphs, lists, etc.

### Verbatim Blocks

Verbatim blocks expose their children as a `VerbatimContainer` and accept
`Vec<VerbatimContent>` in their constructors. Even though only verbatim lines are legal,
callers still get a `Container` that dereferences to `Vec<ContentItem>` for traversal.

## Compile-Fail Guarantees

The `lex-parser` crate now ships `trybuild` tests that prove invalid combinations fail to
compile. For example:

```rust,compile_fail
use lex_parser::lex::ast::elements::typed_content::SessionContent;
use lex_parser::lex::ast::{Definition, Session, TextContent};

fn main() {
    let subject = TextContent::from_string("Term".to_string(), None);
    let session = Session::with_title("Nested".to_string());
    let _definition = Definition::new(subject, vec![SessionContent::Session(session)]);
    // error[E0308]: expected `Vec<ContentElement>`, found `Vec<SessionContent>`
}
```

See `lex-parser/tests/compile_fail` for the full set of examples. Run
`TRYBUILD=overwrite cargo test -p lex-parser --test compile_fail` if you intentionally
change the diagnostics.

## Visitor and Traversal Impact

The visitor APIs still operate on `ContentItem` because traversal needs a uniform enum to
walk heterogeneous trees. Typed content only exists at construction boundaries, so
traversal ergonomics remain unchanged:

- Constructors enforce policy at compile time.
- Containers store `ContentItem` internally for visitor dispatch.
- `ContentElement::try_from` remains available for ad‑hoc conversions inside tests.

If a parser bug violates the policy (e.g., emits a Session under a Definition) the
conversion inside `AstBuilder` will panic, surfacing the bug in debug builds rather than
silently producing a malformed tree.

## Migration Notes

- `Container::new` and the `ContainerNode` alias have been removed—use the typed
  constructors instead.
- Public builder functions (`build_session`, `build_definition`, etc.) now accept typed
  vectors, so downstream code must construct `SessionContent`/`ContentElement` directly.
- Runtime `expect("cannot contain")` checks have been deleted; rely on the type system or
  the compile-fail tests above.

This keeps the container story simple: **if it compiles, it matches the grammar.**
