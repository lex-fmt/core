:: doc.note severity=info :: Document preface.

1. Intro

    Welcome to *Lex* _format_ with `code` and #math# plus references [^source] and [@spec2025 p.4] and [Cache].

    Cache:
        A definition body referencing [Cache].

    :: callout ::
        Session-level annotation body.
    ::

    - Bullet item referencing [42]
    - Nested bullet
        Nested paragraph inside list.

    CLI Example:
        lex build
        lex serve
    :: shell language=bash

:: 42 ::
    Footnote forty two for bullet.
::

:: source ::
    Footnote referenced in text.
::
