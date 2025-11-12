//! # lex
//!
//! A parser for the lex format.
//!
//! File Layout
//!
//! For the time being, and probably at times, we will be running multiple lexer and parser
//! designs side by side. As the code gets more complicated comparing versions is key, and having them
//! side by side makes this easier, including comparision testing. These versions might, as they do
//! now, have different lexer outputs and parser inputs The contract is to have the same global input
//! (the lex source) and the same global output (the AST).
//! But various desings will make different tradeoffs on what gets done in lexing and parsing, so we
//! do not commit to a common lexer or parser outputs. But often different designs do share a
//! significant amount of code.
//!
//! Hence the layout should be:
//! src/lex/parser
//!   ├── linebased    The current parser design
//!   └── <common>     Shared code for AST building and IR

//! So the general form is src/lex/parser|lexer|design|common
//!   
//! For comprehensive testing guidelines, see the [testing module](lex::testing).
//! All parser tests must follow strict rules using verified lex sources and AST assertions.

#![allow(rustdoc::invalid_html_tags)]

pub mod lex;

/// A simple function to demonstrate the library works
pub fn hello() -> &'static str {
    "Hello from lex!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from lex!");
    }
}
