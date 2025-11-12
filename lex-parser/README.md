# lex-parser

The `lex-parser` crate builds typed ASTs for the lex document format. Recent work
(#228–#235) replaced the old dynamic container model with policy-driven containers that
encode nesting rules at the type level.

## Typed Containers

Each container constructor now accepts a typed vector:

- `Session::new(title, Vec<SessionContent>)`
- `Definition::new(subject, Vec<ContentElement>)`
- `Annotation::new(label, params, Vec<ContentElement>)`
- `List::new(Vec<ListItem>)`
- `Verbatim::new(subject, Vec<VerbatimContent>, closing_annotation)`

If code compiles, it satisfies the nesting rules. Invalid combinations fail to compile,
as demonstrated by the `trybuild` tests under `tests/compile_fail`.

```rust,compile_fail
use lex_parser::lex::ast::elements::typed_content::SessionContent;
use lex_parser::lex::ast::{Definition, Session, TextContent};

fn main() {
    let subject = TextContent::from_string("Term".into(), None);
    let session = Session::with_title("Nested".into());
    let _definition = Definition::new(subject, vec![SessionContent::Session(session)]);
}
```

Run the tests to see the diagnostic:

```bash
TRYBUILD=overwrite cargo test -p lex-parser --test compile_fail
```

For a deeper architectural overview read
[`docs/architecture/type-safe-containers.md`](../docs/architecture/type-safe-containers.md).
