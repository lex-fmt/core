This is the repo for the lex format. And we will build it incrementally, focussing on the three core types at first, and structure. These are called the trifecta: sessions, paragraphs and lists.

Note the general tone and style of the docuementation: 
- All lex formatted (must dogfood)
- Straight, objective and informative. No emojis, no marketing speak of benefits or all tests passing
- Simple: this is not a mission critical high throughput format, but a presonal project, hence a 400 page spec is out of the question.

You will see mentions of things ,like the lex-marker, on the grammar that sees useless, as the none of the current elementns use it. These should be taken into account and not ignored, because factor them in now makes a lot of things easier then later (i.e. tokenization changes)

Documentation Structure.

    specs/<verions>: each language verions is stored in it's own directory, as we will be building up on final language by incrementaly building the specs in versions. Having versions side by side is useful  as many of the tasks will be about adding support for language additions and, doing so is much easier comparig the new verions. 

    Inside each version: 
        1. general.lex -> A introduction to the format, which includes general points,like character encoding a description of each major element type.
        2. grammar.lex -> the syntax and grammar/syntax defs for the language, using the simplified BNF-like described in the document

Working with Markdown input.

    The markdown importer lives in lex-babel and is powered by the comrak crate. We keep
    reference fixtures under lex-babel/tests/fixtures/, including copies of the CommonMark
    and Comrak READMEs with attribution headers. The importer converts Markdown → IR →
    Lex AST before being consumed by other formats such as the tag/treeviz visualizers.

    Recommended workflows:
        - Run `cargo test -p lex-babel markdown::import` to exercise the element/unit suites.
        - Run `cargo test -p lex-cli` to ensure the CLI can ingest Markdown inputs.
        - Inspect Markdown on the shell via `cargo run --bin lex -- path/to/doc.md --to tag`
          (auto-detected `--from markdown`).

    The CLI test + command above mirror the manual debugging flow mentioned in docs/dev/guides.
